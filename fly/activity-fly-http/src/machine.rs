use crate::generated::exports::obelisk_flyio::activity_fly_http::machines::{
    ExecConfig, ExecResponse, Guest, Machine, MachineConfig,
};
use crate::generated::obelisk_flyio::activity_fly_http::regions::Region;
use crate::machine::ser::MachineConfigSer;
use crate::wstd_util::JsonRequest as _;
use crate::{API_BASE_URL, AppName, Component, MachineId, request_with_api_token};
use anyhow::{Context, anyhow, bail, ensure};
use ser::{
    ExecResponseSer, MachineCreateRequestSer, MachineCreateResponseSer, MachineUpdateRequestSer,
    ResponseErrorSer,
};
use serde::Serialize;
use wstd::http::{Body, Client, Method, StatusCode};
use wstd::runtime::block_on;

pub(crate) mod ser {
    use std::collections::HashMap;

    use crate::generated::exports::obelisk_flyio::activity_fly_http::machines::{
        ExecResponse, FileConfig, GuestConfig, InitConfig, MachineConfig, MachineRestart, Mount,
        ServiceConfig, StopConfig,
    };
    use crate::generated::obelisk_flyio::activity_fly_http::regions::Region;
    use serde::{Deserialize, Serialize};

    // Fix env var serialization.
    // WIT: `option<list<tuple<string, string>>>`
    // Expected `{"key1":"val1",...}`
    // TODO: Remove when `map` type is implemented.
    #[derive(Serialize, Debug)]
    pub(crate) struct MachineConfigSer {
        pub image: String,
        pub guest: Option<GuestConfig>,
        /// Destroy the VM after first exec
        pub auto_destroy: Option<bool>,
        pub init: Option<InitConfig>,
        pub env: Option<HashMap<String, String>>,
        pub restart: Option<MachineRestart>,
        pub stop_config: Option<StopConfig>,
        pub mounts: Option<Vec<Mount>>,
        pub services: Option<Vec<ServiceConfig>>,
        pub files: Option<Vec<FileConfig>>,
    }

    impl From<MachineConfig> for MachineConfigSer {
        fn from(value: MachineConfig) -> Self {
            MachineConfigSer {
                image: value.image,
                guest: value.guest,
                auto_destroy: value.auto_destroy,
                init: value.init,
                env: value.env.map(HashMap::from_iter),
                restart: value.restart,
                stop_config: value.stop_config,
                mounts: value.mounts,
                services: value.services,
                files: value.files,
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub(crate) struct MachineCreateRequestSer {
        pub(crate) name: String,
        pub(crate) config: MachineConfigSer,
        pub(crate) region: Option<Region>,
    }

    #[derive(Serialize, Debug)]
    pub(crate) struct MachineUpdateRequestSer {
        pub(crate) config: MachineConfigSer,
        pub(crate) region: Option<Region>,
    }

    #[derive(Deserialize)]
    pub(crate) struct MachineCreateResponseSer {
        pub(crate) id: String,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct ResponseErrorSer {
        error: String,
    }

    impl ResponseErrorSer {
        pub(crate) fn get_machine_id_on_creation_conflict(&self) -> Option<&str> {
            const PREFIX: &str = "already_exists: unique machine name violation, machine ID ";
            const SUFFIX: &str = " already exists with name ";
            if let Some(0) = self.error.find(PREFIX) {
                let start_idx = PREFIX.len();
                if let Some(end_idx) = self.error[start_idx..].find(SUFFIX) {
                    return Some(&self.error[start_idx..start_idx + end_idx]);
                }
            }
            None
        }
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct ExecResponseSer {
        exit_code: Option<i32>,
        exit_signal: Option<i32>,
        stderr: Option<String>,
        stdout: Option<String>,
    }
    impl From<ExecResponseSer> for ExecResponse {
        fn from(value: ExecResponseSer) -> Self {
            ExecResponse {
                exit_code: value.exit_code,
                exit_signal: value.exit_signal,
                stderr: value.stderr,
                stdout: value.stdout,
            }
        }
    }
}

async fn list(app_name: AppName) -> Result<Vec<Machine>, anyhow::Error> {
    let url = format!("{API_BASE_URL}/apps/{app_name}/machines");
    let request = request_with_api_token()?
        .method(Method::GET)
        .uri(url)
        .body(Body::empty())?;
    let response = Client::new().send(request).await?;
    let resp_status = response.status();
    let mut response = response.into_body();
    let response = response.str_contents().await?;

    if resp_status.is_success() {
        let response: Vec<Machine> = serde_json::from_str(response)
            .inspect_err(|_| eprintln!("cannot deserialize: {response}"))?;
        Ok(response)
    } else {
        eprintln!("Got error status {resp_status}");
        Err(anyhow!("failed with status {resp_status}: {response}"))
    }
}

async fn get(app_name: AppName, machine_id: MachineId) -> Result<Option<Machine>, anyhow::Error> {
    let url = format!("{API_BASE_URL}/apps/{app_name}/machines/{machine_id}");
    let request = request_with_api_token()?
        .method(Method::GET)
        .uri(url)
        .body(Body::empty())?;
    let response = Client::new().send(request).await?;
    let resp_status = response.status();
    let mut response = response.into_body();
    let response = response.str_contents().await?;

    if resp_status.is_success() {
        let response: Machine = serde_json::from_str(response)
            .inspect_err(|_| eprintln!("cannot deserialize: {response}"))?;
        Ok(Some(response))
    } else if resp_status == StatusCode::NOT_FOUND {
        Ok(None)
    } else {
        eprintln!("Got error status {resp_status}");
        Err(anyhow!("failed with status {resp_status}: {response}"))
    }
}

async fn create(
    app_name: AppName,
    machine_name: String,
    machine_config: MachineConfig,
    region: Option<Region>,
) -> Result<String, anyhow::Error> {
    {
        let request_payload = MachineCreateRequestSer {
            name: machine_name,
            config: MachineConfigSer::from(machine_config),
            region,
        };
        let url = format!("{API_BASE_URL}/apps/{app_name}/machines");
        let request = request_with_api_token()?
            .method(Method::POST)
            .uri(url)
            .json(&request_payload)?;

        let response = Client::new().send(request).await?;
        let resp_status = response.status();
        let mut response = response.into_body();
        let response = response.str_contents().await?;

        if resp_status.is_success() {
            let resp: MachineCreateResponseSer = serde_json::from_str(response)
                .with_context(|| format!("Deserialization of response failed: `{response}`"))?;
            return Ok(resp.id);
        }
        eprintln!("Got error status {resp_status}");
        if resp_status == StatusCode::CONFLICT {
            let error: ResponseErrorSer = serde_json::from_str(response)
                .with_context(|| format!("cannot parse error response: `{response}`"))?;
            let machine_id = error.get_machine_id_on_creation_conflict().with_context(
                || "machine id cannot be parsed from 409 error response: `{error:?}`",
            )?;
            Ok(machine_id.to_string())
        } else {
            Err(anyhow!("{resp_status} - {response}"))
        }
    }
}

async fn update(
    app_name: AppName,
    machine_id: MachineId,
    machine_config: MachineConfig,
    region: Option<Region>,
) -> Result<(), anyhow::Error> {
    {
        let request_payload = MachineUpdateRequestSer {
            config: MachineConfigSer::from(machine_config),
            region,
        };
        let url = format!("{API_BASE_URL}/apps/{app_name}/machines/{machine_id}");
        let request = request_with_api_token()?
            .method(Method::POST)
            .uri(url)
            .json(&request_payload)?;

        let response = Client::new().send(request).await?;
        let resp_status = response.status();
        let mut response = response.into_body();
        let response = response.str_contents().await?;

        if resp_status.is_success() {
            let resp: MachineCreateResponseSer = serde_json::from_str(response)
                .with_context(|| format!("Deserialization of response failed: `{response}`"))?;
            ensure!(
                resp.id == machine_id.as_ref(),
                "unexpected id returned, expected {machine_id} got {id}",
                id = resp.id
            );
            return Ok(());
        }
        bail!("{resp_status} - {response}")
    }
}

async fn exec(
    app_name: AppName,
    machine_id: MachineId,
    command: Vec<String>,
    config: ExecConfig,
) -> Result<ExecResponse, anyhow::Error> {
    #[derive(Serialize)]
    struct ExecPayload {
        command: Vec<String>,
        timeout: Option<u16>,
        stdin: Option<String>,
    }
    let url = format!("{API_BASE_URL}/apps/{app_name}/machines/{machine_id}/exec");
    let body = ExecPayload {
        command,
        timeout: config.timeout_secs,
        stdin: config.stdin,
    };
    let request = request_with_api_token()?
        .method(Method::POST)
        .uri(url)
        .json(&body)?;
    let response = Client::new().send(request).await?;
    let resp_status = response.status();
    let mut response = response.into_body();
    let response = response.str_contents().await?;

    if resp_status.is_success() {
        let response: ExecResponseSer = serde_json::from_str(response)
            .inspect_err(|_| eprintln!("cannot deserialize: {response}"))?;
        Ok(response.into())
    } else {
        eprintln!("Got error status {resp_status}");
        Err(anyhow!("failed with status {resp_status}: {response}"))
    }
}

async fn change_machine(
    app_name: AppName,
    machine_id: MachineId,
    url_suffix: &'static str,
) -> Result<(), anyhow::Error> {
    let url = format!("{API_BASE_URL}/apps/{app_name}/machines/{machine_id}/{url_suffix}");
    send_request(url, Method::POST).await
}

async fn delete(
    app_name: AppName,
    machine_id: MachineId,
    force: bool,
) -> Result<(), anyhow::Error> {
    let url = format!("{API_BASE_URL}/apps/{app_name}/machines/{machine_id}?force={force}");
    send_request(url, Method::DELETE).await
}

async fn send_request(url: String, method: Method) -> Result<(), anyhow::Error> {
    let request = request_with_api_token()?
        .method(method)
        .uri(url)
        .body(Body::empty())?;

    let response = Client::new().send(request).await?;
    let resp_status = response.status();
    let mut response = response.into_body();
    let response = response.str_contents().await?;

    if resp_status.is_success() {
        Ok(())
    } else {
        Err(anyhow!("failed with status {resp_status}: {response}",))
    }
}

// Implementation of the vm interface for the component.
impl Guest for Component {
    fn list(app_name: String) -> Result<Vec<Machine>, String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            block_on(list(app_name))
        })()
        .map_err(|err| err.to_string())
    }

    fn get(app_name: String, machine_id: String) -> Result<Option<Machine>, String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(get(app_name, machine_id))
        })()
        .map_err(|err| err.to_string())
    }

    fn create(
        app_name: String,
        machine_name: String,
        machine_config: MachineConfig,
        region: Option<Region>,
    ) -> Result<String, String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            block_on(create(app_name, machine_name, machine_config, region))
        })()
        .map_err(|err| err.to_string())
    }

    fn update(
        app_name: String,
        machine_id: String,
        machine_config: MachineConfig,
        region: Option<Region>,
    ) -> Result<(), String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(update(app_name, machine_id, machine_config, region))
        })()
        .map_err(|err| err.to_string())
    }

    fn stop(app_name: String, machine_id: String) -> Result<(), String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(change_machine(app_name, machine_id, "stop"))
        })()
        .map_err(|err| err.to_string())
    }

    fn suspend(app_name: String, machine_id: String) -> Result<(), String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(change_machine(app_name, machine_id, "suspend"))
        })()
        .map_err(|err| err.to_string())
    }

    fn start(app_name: String, machine_id: String) -> Result<(), String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(change_machine(app_name, machine_id, "start"))
        })()
        .map_err(|err| err.to_string())
    }

    fn restart(app_name: String, machine_id: String) -> Result<(), String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(change_machine(app_name, machine_id, "restart"))
        })()
        .map_err(|err| err.to_string())
    }

    fn delete(app_name: String, machine_id: String, force: bool) -> Result<(), String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(delete(app_name, machine_id, force))
        })()
        .map_err(|err| err.to_string())
    }

    fn exec(
        app_name: String,
        machine_id: String,
        command: Vec<String>,
        config: ExecConfig,
    ) -> Result<ExecResponse, String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(exec(app_name, machine_id, command, config))
        })()
        .map_err(|err| err.to_string())
    }

    fn exec_check_success(
        app_name: String,
        machine_id: String,
        command: Vec<String>,
        config: ExecConfig,
    ) -> Result<ExecResponse, String> {
        (|| {
            let app_name = AppName::new(app_name)?;
            let machine_id = MachineId::new(machine_id)?;
            block_on(async {
                let resp = exec(app_name, machine_id, command, config).await?;
                if resp.exit_code == Some(0) {
                    Ok(resp)
                } else {
                    bail!("non-successful exit status - {resp:?}")
                }
            })
        })()
        .map_err(|err| err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::ser::ResponseErrorSer;
    use crate::generated::{
        exports::obelisk_flyio::activity_fly_http::machines::Machine,
        obelisk_flyio::activity_fly_http::regions::Region,
    };
    use insta::assert_debug_snapshot;
    use serde_json::json;

    #[test]
    fn region_ser() {
        assert_eq!("\"ams\"", serde_json::to_string(&Region::Ams).unwrap());
    }

    #[test]
    fn region_de() {
        assert_matches::assert_matches!(serde_json::from_str("\"ams\"").unwrap(), Region::Ams);
    }

    #[test]
    fn get_machine_id_on_creation_conflict_should_work() {
        let response = json!({"error": "already_exists: unique machine name violation, machine ID 32876249a30918 already exists with name \"foo\""});
        let response: ResponseErrorSer = serde_json::from_value(response).unwrap();
        let id = response.get_machine_id_on_creation_conflict().unwrap();
        assert_eq!("32876249a30918", id);
    }

    #[test]
    fn machine_deserialization() {
        let json = r#"
{
    "id": "d892d51f691378",
    "name": "obelisk",
    "state": "created",
    "region": "ams",
    "instance_id": "01KN183BZYXG5QCDBEPMFHE84Y",
    "private_ip": "fdaa:0:fcc8:a7b:624:3e5c:9e8d:2",
    "config": {
        "init": {
            "entrypoint": [
                "/usr/bin/env",
                "bash",
                "/usr/local/bin/litestream-entrypoint.sh"
            ],
            "swap_size_mb": 256
        },
        "guest": {
            "cpu_kind": "shared",
            "cpus": 1,
            "memory_mb": 256
        },
        "mounts": [
            {
                "encrypted": true,
                "path": "/volume",
                "size_gb": 1,
                "volume": "vol_rk19ee7jnp5dmpk4",
                "name": "db"
            }
        ],
        "services": [
            {
                "protocol": "tcp",
                "internal_port": 9091,
                "ports": [
                    {
                        "port": 444,
                        "handlers": [
                            "tls"
                        ]
                    }
                ],
                "force_instance_key": null
            },
            {
                "protocol": "tcp",
                "internal_port": 9090,
                "ports": [
                    {
                        "port": 443,
                        "handlers": [
                            "tls"
                        ]
                    }
                ],
                "force_instance_key": null
            }
        ],
        "image": "getobelisk/obelisk:0.36.1-ubuntu-litestream",
        "files": [
            {
                "guest_path": "/etc/obelisk/obelisk.toml",
                "raw_value": "W1thY3Rpdml0eV93YXNtXV0KZW52X3ZhcnMgPSBbIkZMWV9BUElfVE9LRU4iXQpsb2NhdGlvbiA9ICJvY2k6Ly9kb2NrZXIuaW8vZ2V0b2JlbGlzay9jb21wb25lbnRzX2ZseV9hY3Rpdml0eV9mbHlfaHR0cDoyMDI2LTAzLTMwQHNoYTI1NjpmZWYwNzUzMTEyMGUxMGE4ZTU5ZWRiYjQ5MjdkY2ZmZmFlNjhhZWQ2OTI1ZDc2ZWE0M2M4ZTAyZTMwNTg0MTYxIgptYXhfcmV0cmllcyA9IDYKbmFtZSA9ICJhY3Rpdml0eV9mbHlfaHR0cCIKCltbYWN0aXZpdHlfd2FzbS5hbGxvd2VkX2hvc3RdXQptZXRob2RzID0gIioiCnBhdHRlcm4gPSAiKjovLyo6KiIKClthY3Rpdml0eV93YXNtLmV4ZWMubG9ja19leHBpcnldCnNlY29uZHMgPSAxNQoKW1thY3Rpdml0eV93YXNtXV0KbG9jYXRpb24gPSAib2NpOi8vZG9ja2VyLmlvL2dldG9iZWxpc2svY29tcG9uZW50c19odHRwX2FjdGl2aXR5X2h0dHBfZ2VuZXJpYzoyMDI2LTAzLTMwQHNoYTI1NjpkMTcwMDkyZjMwMjMzODA5NjBjNDU5YmI5MmQ3MWI2YTBhYzhjOGQzZjE2Y2NiODYzZWVkNzJlNjljZjcxNTdkIgpuYW1lID0gImFjdGl2aXR5X2h0dHBfZ2VuZXJpYyIKCltbYWN0aXZpdHlfd2FzbS5hbGxvd2VkX2hvc3RdXQptZXRob2RzID0gIioiCnBhdHRlcm4gPSAiKjovLyo6KiIKClthY3Rpdml0eV93YXNtLmV4ZWMubG9ja19leHBpcnldCnNlY29uZHMgPSA1CgpbW2FjdGl2aXR5X3dhc21dXQpsb2NhdGlvbiA9ICJvY2k6Ly9kb2NrZXIuaW8vZ2V0b2JlbGlzay9jb21wb25lbnRzX29iZWxpc2tfYWN0aXZpdHlfb2JlbGlza19jbGllbnRfaHR0cDoyMDI2LTAzLTMwQHNoYTI1NjpmNWZhMjgyYmI2OTQxYjM1MjBjOWFjOThhOTQ5ZjBjMTQzOGViM2Y3MjgzNjk4NTgxNjViYmE0YTkxYThkZDRjIgpuYW1lID0gImFjdGl2aXR5X29iZWxpc2tfY2xpZW50IgoKW1thY3Rpdml0eV93YXNtLmFsbG93ZWRfaG9zdF1dCm1ldGhvZHMgPSAiKiIKcGF0dGVybiA9ICIqOi8vKjoqIgoKW1t3ZWJob29rX2VuZHBvaW50X3dhc21dXQpodHRwX3NlcnZlciA9ICJoZWFsdGhjaGVja19zZXJ2ZXIiCmxvY2F0aW9uID0gIm9jaTovL2RvY2tlci5pby9nZXRvYmVsaXNrL2NvbXBvbmVudHNfZmx5aW9fd2ViaG9va19oZWFsdGhjaGVjazoyMDI2LTAzLTMwQHNoYTI1NjphODExOTljN2RmNGQyODI3MjVlZDU3ZjBkMjBmNDIxNGE2NGRjMDRhYTgzMjA5MTFjZGExNzQ4ZGNmZGU2OGY4IgpuYW1lID0gIndlYmhvb2tfaGVhbHRoY2hlY2siCnJvdXRlcyA9IFsiIl0KCltbd29ya2Zsb3dfd2FzbV1dCmxvY2F0aW9uID0gIm9jaTovL2RvY2tlci5pby9nZXRvYmVsaXNrL2NvbXBvbmVudHNfZmx5aW9fb2JlbGlza19kZXBsb3llcl9mbHlpbzoyMDI2LTAzLTMwQHNoYTI1NjowN2I1ZjI0OTJhM2Y0MWEzNmZiZTc0Y2RhODAxYTdhMGMyOTFhOTg2Zjk5ZDBjMWQwODY2YzQzYmY4OWQ3YWEyIgpuYW1lID0gIm9iZWxpc2tfZGVwbG95ZXJfZmx5aW8iCg=="
            },
            {
                "guest_path": "/etc/obelisk/server.toml",
                "raw_value": "W2FwaV0KbGlzdGVuaW5nX2FkZHIgPSAiWzo6XTo1MDA1IgoKW2RhdGFiYXNlLnNxbGl0ZV0KZGlyZWN0b3J5ID0gIi92b2x1bWUvb2JlbGlzay1zcWxpdGUiCgpbZGF0YWJhc2Uuc3FsaXRlLnByYWdtYV0KY2FjaGVfc2l6ZSA9ICIzMDAwIgoKW1todHRwX3NlcnZlcl1dCmxpc3RlbmluZ19hZGRyID0gIjAuMC4wLjA6OTA5MSIKbmFtZSA9ICJoZWFsdGhjaGVja19zZXJ2ZXIiCgpbbG9nLmZpbGVdCmRpcmVjdG9yeSA9ICIvdmFyL2xvZyIKZW5hYmxlZCA9IHRydWUKcHJlZml4ID0gIm9iZWxpc2subG9nIgp0YXJnZXQgPSB0cnVlCgpbd2FzbV0KY2FjaGVfZGlyZWN0b3J5ID0gIi92b2x1bWUvd2FzbSIKcGFyYWxsZWxfY29tcGlsYXRpb24gPSBmYWxzZQoKW3dhc20uY29kZWdlbl9jYWNoZV0KZGlyZWN0b3J5ID0gIi92b2x1bWUvY29kZWdlbiIKClt3ZWJ1aV0KbGlzdGVuaW5nX2FkZHIgPSAiWzo6XTo4MDgwIgo="
            },
            {
                "guest_path": "/usr/local/bin/litestream-entrypoint.sh",
                "raw_value": "CiMhL3Vzci9iaW4vZW52IGJhc2gKCnNldCAtZXVvIHBpcGVmYWlsCgpsaXRlc3RyZWFtIHJlc3RvcmUgLWlmLXJlcGxpY2EtZXhpc3RzIC0tY29uZmlnIC9ldGMvbGl0ZXN0cmVhbS55bWwgL3ZvbHVtZS9vYmVsaXNrLXNxbGl0ZS9vYmVsaXNrLnNxbGl0ZQpleGVjIGxpdGVzdHJlYW0gcmVwbGljYXRlIC0tY29uZmlnIC9ldGMvbGl0ZXN0cmVhbS55bWwgLS1leGVjICdvYmVsaXNrIHNlcnZlciBydW4gLS1zZXJ2ZXItY29uZmlnIC9ldGMvb2JlbGlzay9zZXJ2ZXIudG9tbCAtLWRlcGxveW1lbnQgL2V0Yy9vYmVsaXNrL29iZWxpc2sudG9tbCcKICAgICAgICA="
            },
            {
                "guest_path": "/etc/litestream.yml",
                "raw_value": "CmRiczoKICAtIHBhdGg6ICIvdm9sdW1lL29iZWxpc2stc3FsaXRlL29iZWxpc2suc3FsaXRlIgogICAgcmVwbGljYToKICAgICAgdXJsOiAiczM6Ly9saXRlc3RyZWFtLWJ1Y2tldC40OGU2NWVkZjc1OTQzOC52bS5pdHNlbGYuaW50ZXJuYWw6OTAwMC9saXRlc3RyZWFtL29iZWxpc2siCiAgICAgIGFjY2Vzcy1rZXktaWQ6ICJtaW5pb2FkbWluIgogICAgICBzZWNyZXQtYWNjZXNzLWtleTogIm1pbmlvYWRtaW4iCg=="
            }
        ],
        "restart": {
            "policy": "on-failure",
            "max_retries": 5
        }
    },
    "incomplete_config": null,
    "image_ref": {
        "registry": "docker-hub-mirror.fly.io",
        "repository": "getobelisk/obelisk",
        "tag": "0.36.1-ubuntu-litestream",
        "digest": "sha256:4c78dcb79c7435c460a1f988f8beb4787e595dbb10456642deff70c613c31894",
        "labels": {
            "org.opencontainers.image.ref.name": "ubuntu",
            "org.opencontainers.image.version": "24.04"
        }
    },
    "created_at": "2026-03-31T06:08:08Z",
    "updated_at": "2026-03-31T06:08:08Z",
    "events": [
        {
            "id": "01KN183C327XW9SZYH8P8E3D16",
            "type": "launch",
            "status": "created",
            "source": "user",
            "timestamp": 1774937288802
        },
        {
            "id": "01KN183BZZKK58758ZWX292X93",
            "type": "launch",
            "status": "pending",
            "source": "flyd",
            "timestamp": 1774937288703
        }
    ],
    "host_status": "ok"
}
        "#;
        let machine: Machine = serde_json::from_str(json).unwrap();
        assert_debug_snapshot!(machine)
    }
}
