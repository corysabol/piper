pub mod agent_proto {
    tonic::include_proto!("piper");
}

// Proto generated client
use agent_proto::piper_agent_client::PiperAgentClient;

use agent_proto::PipelineInput;

pub async fn client_run(
    agent_addr: String,
    path: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running remote pipeline!");
    // Connect to server
    // User server add if given, otherwise use default
    let mut client = PiperAgentClient::connect(agent_addr).await?;

    // load the pipeline file contents
    let pipeline_string = std::fs::read_to_string(path).unwrap();
    let request = tonic::Request::new(PipelineInput {
        pipeline: pipeline_string,
    });

    let response = client.run_pipeline(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
