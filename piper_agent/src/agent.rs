use tonic::{transport::Server, Request, Response, Status};
// Import the generated rust code into module
mod agent_proto {
    tonic::include_proto!("piper");
}
// Proto generated server traits
use agent_proto::piper_agent_server::{PiperAgent, PiperAgentServer};
// Proto message structs
use agent_proto::{PipelineInput, PipelineOutput};

use piper_runner::*;
use piper_tasks::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::runtime::Runtime;

use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyBytes, PyDict, PyModule},
};

pub struct AgentOptions {
    listen_addr: String,
}

#[derive(Default)]
pub struct Agent {}

#[tonic::async_trait]
impl PiperAgent for Agent {
    async fn run_pipeline(
        &self,
        request: Request<PipelineInput>,
    ) -> Result<Response<PipelineOutput>, Status> {
        // Need to run without a file path
        let req_pipeline = request.into_inner();
        let pipeline = req_pipeline.pipeline;

        runner::run(pipeline);
        Ok(Response::new(PipelineOutput {
            output: "".to_string(),
        }))
    }
}

pub async fn start_agent(listen_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = listen_addr.parse().unwrap();
    let agent = Agent::default();

    println!("Piper remote agent listening on {}", addr);

    Server::builder()
        .add_service(PiperAgentServer::new(agent))
        .serve(addr)
        .await?;

    Ok(())
}
