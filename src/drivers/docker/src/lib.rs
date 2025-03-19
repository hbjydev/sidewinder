use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bollard::{
    container::{Config, CreateContainerOptions},
    image::CreateImageOptions,
    secret::{ContainerCreateResponse, HostConfig, Mount, MountTypeEnum, Volume},
    volume::CreateVolumeOptions,
    Docker as BoDocker,
};
use futures_util::TryStreamExt;
use sidewinder_core::{
    exec::{driver::Driver, Execution},
    server::ServerConfig,
};
use tokio::sync::Mutex;

const IMAGE: &str = "kexanone/reforger-server:latest";

pub struct Docker {
    connection: Mutex<BoDocker>,
}

impl Docker {
    pub fn new() -> Result<Self> {
        let conn = BoDocker::connect_with_socket_defaults()?;
        Ok(Self {
            connection: Mutex::new(conn),
        })
    }
}

impl Driver for Docker {
    #[tracing::instrument(skip(self, config))]
    async fn run_server(&self, id: String, config: ServerConfig) -> Result<Execution> {
        let conn = self.connection.lock().await;

        let config_path = config.write_to_file(id.clone())?;

        ensure_image(&conn).await?;

        let volumes = ensure_volumes(&conn, id.clone()).await?;

        let container_config = get_container_config(config.clone(), volumes, config_path);
        let container = reset_container_config(&conn, id.clone(), container_config).await?;

        conn.start_container::<String>(&container.id, None).await?;
        tracing::info!(
            "container for server {} started with id {}",
            id,
            container.id
        );

        Ok(Execution {
            id: container.id,
            server_id: id.clone(),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn list_servers(&self) -> Result<Vec<Execution>> {
        let _conn = self.connection.lock().await;
        Ok(Vec::new())
    }

    #[tracing::instrument(skip(self, exec))]
    async fn stop_server(&self, exec: Execution) -> Result<()> {
        let conn = self.connection.lock().await;
        conn.stop_container(&exec.id, None).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, _exec))]
    async fn get_logs(&self, _exec: Execution) -> Result<()> {
        Ok(())
    }
}

#[tracing::instrument]
async fn ensure_image(conn: &BoDocker) -> Result<()> {
    conn.create_image(
        Some(CreateImageOptions {
            from_image: IMAGE,
            ..Default::default()
        }),
        None,
        None,
    )
    .try_collect::<Vec<_>>()
    .await?;

    Ok(())
}

fn get_create_container_opts(id: String) -> CreateContainerOptions<String> {
    let opts = CreateContainerOptions {
        name: format!("swndr-{}", id),
        ..Default::default()
    };

    opts
}

#[tracing::instrument]
async fn get_or_create_volume(conn: &BoDocker, id: String, name: &str) -> Result<Volume> {
    let vol_name = format!("swndr-{id}-{name}");
    let vol = conn.inspect_volume(&vol_name).await;

    match vol {
        Ok(volume) => Ok(volume),
        Err(err) => match err {
            bollard::errors::Error::DockerResponseServerError {
                status_code,
                message,
            } => match status_code {
                404 => {
                    // Create a new volume
                    let vol = conn
                        .create_volume(CreateVolumeOptions {
                            name: vol_name,
                            ..Default::default()
                        })
                        .await?;

                    Ok(vol)
                }
                _ => Err(anyhow!("failed to get docker volumes: {}", message)),
            },
            err => Err(err.into()),
        },
    }
}

struct ServerVolumes(Volume);
impl ServerVolumes {
    pub fn new(profile: Volume) -> Self {
        Self(profile)
    }

    pub fn get_profile(&self) -> Volume {
        self.0.clone()
    }
}

async fn ensure_volumes(conn: &BoDocker, id: String) -> Result<ServerVolumes> {
    let profile = get_or_create_volume(&conn, id.clone(), "profile").await?;

    Ok(ServerVolumes::new(profile))
}

fn get_container_config(
    _cfg: ServerConfig,
    volumes: ServerVolumes,
    config_secret_path: PathBuf,
) -> Config<String> {
    Config {
        image: Some(IMAGE.to_string()),
        host_config: Some(HostConfig {
            mounts: Some(vec![
                Mount {
                    target: Some(String::from("/reforger/profile")),
                    source: Some(volumes.get_profile().name),
                    typ: Some(MountTypeEnum::VOLUME),
                    read_only: Some(false),
                    ..Default::default()
                },
                Mount {
                    target: Some(String::from("/reforger/config.json")),
                    // FIXME: Don't unwrap this
                    source: Some(String::from(config_secret_path.to_str().unwrap())),
                    typ: Some(MountTypeEnum::BIND),
                    read_only: Some(false),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }),
        cmd: Some(vec![
            String::from("-config"),
            String::from("/reforger/config.json"),
            String::from("-profile"),
            String::from("/reforger/profile"),
            String::from("-maxFPS"),
            String::from("60"),
        ]),
        ..Default::default()
    }
}

async fn create_container(
    conn: &BoDocker,
    id: String,
    container_config: Config<String>,
) -> Result<ContainerCreateResponse> {
    let container = conn
        .create_container(Some(get_create_container_opts(id)), container_config)
        .await?;

    Ok(container)
}

async fn reset_container_config(
    conn: &BoDocker,
    id: String,
    container_config: Config<String>,
) -> Result<ContainerCreateResponse> {
    let container_name = format!("swndr-{}", id);
    let container = conn.inspect_container(&container_name, None).await;

    match container {
        Ok(_) => {
            // Recreate the container
            conn.stop_container(&container_name, None).await?;
            conn.remove_container(&container_name, None).await?;
            create_container(&conn, id, container_config).await
        }

        Err(err) => match err {
            // Create the container for the first time
            bollard::errors::Error::DockerResponseServerError {
                status_code,
                message,
            } => match status_code {
                404 => create_container(&conn, id, container_config).await,
                _ => Err(anyhow!("failed to get docker container: {}", message)),
            },
            err => Err(err.into()),
        },
    }
}
