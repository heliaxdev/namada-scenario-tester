
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::json;

#[derive(Clone, Debug, Deserialize)]
pub enum ProposalType {
    Empty,
    Wasm(PathBuf),
    PgfStewardProposal,
    PgfFundingProposal
}


pub struct ValidProposal {
    pub source: String,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub grace_epoch: u64,
    pub proposal_data: Option<Vec<u8>>,
}
impl ValidProposal {
    pub fn new(source: String, start_epoch: u64, end_epoch: u64, grace_epoch: u64, proposal_type: ProposalType) -> Self {
        let proposal_data = match proposal_type {
            ProposalType::Empty => None,
            ProposalType::Wasm(path) => Some(std::fs::read(path).unwrap()),
            ProposalType::PgfStewardProposal => None,
            ProposalType::PgfFundingProposal => None,
        };
        Self {
            source,
            start_epoch,
            end_epoch,
            grace_epoch,
            proposal_data,
        }
    }

    pub fn generate_proposal(&self) -> serde_json::Value {
        
        if let Some(data) = &self.proposal_data {
            let proposal = json!({
                "proposal": {
                    "content": {
                        "title": "TheTitle",
                        "authors": "test@test.com",
                        "discussions-to": "www.github.com/anoma/aip/1",
                        "created": "2022-03-10T08:54:37Z",
                        "license": "MIT",
                        "abstract": "Ut convallis eleifend orci vel venenatis. Duis vulputate metus in lacus sollicitudin vestibulum. Suspendisse vel velit ac est consectetur feugiat nec ac urna. Ut faucibus ex nec dictum fermentum. Morbi aliquet purus at sollicitudin ultrices. Quisque viverra varius cursus. Praesent sed mauris gravida, pharetra turpis non, gravida eros. Nullam sed ex justo. Ut at placerat ipsum, sit amet rhoncus libero. Sed blandit non purus non suscipit. Phasellus sed quam nec augue bibendum bibendum ut vitae urna. Sed odio diam, ornare nec sapien eget, congue viverra enim.",
                        "motivation": "Ut convallis eleifend orci vel venenatis. Duis vulputate metus in lacus sollicitudin vestibulum. Suspendisse vel velit ac est consectetur feugiat nec ac urna. Ut faucibus ex nec dictum fermentum. Morbi aliquet purus at sollicitudin ultrices.",
                        "details": "Ut convallis eleifend orci vel venenatis. Duis vulputate metus in lacus sollicitudin vestibulum. Suspendisse vel velit ac est consectetur feugiat nec ac urna. Ut faucibus ex nec dictum fermentum. Morbi aliquet purus at sollicitudin ultrices. Quisque viverra varius cursus. Praesent sed mauris gravida, pharetra turpis non, gravida eros.",
                        "requires": "2"
                    },
                    "author": self.source,
                    "voting_start_epoch": self.start_epoch,
                    "voting_end_epoch": self.end_epoch,
                    "grace_epoch": self.grace_epoch
                },
                "data": data
            });
        proposal        
        }
        else {
            let proposal = json!({
                "proposal": {
                    "content": {
                        "title": "TheTitle",
                        "authors": "test@test.com",
                        "discussions-to": "www.github.com/anoma/aip/1",
                        "created": "2022-03-10T08:54:37Z",
                        "license": "MIT",
                        "abstract": "Ut convallis eleifend orci vel venenatis. Duis vulputate metus in lacus sollicitudin vestibulum. Suspendisse vel velit ac est consectetur feugiat nec ac urna. Ut faucibus ex nec dictum fermentum. Morbi aliquet purus at sollicitudin ultrices. Quisque viverra varius cursus. Praesent sed mauris gravida, pharetra turpis non, gravida eros. Nullam sed ex justo. Ut at placerat ipsum, sit amet rhoncus libero. Sed blandit non purus non suscipit. Phasellus sed quam nec augue bibendum bibendum ut vitae urna. Sed odio diam, ornare nec sapien eget, congue viverra enim.",
                        "motivation": "Ut convallis eleifend orci vel venenatis. Duis vulputate metus in lacus sollicitudin vestibulum. Suspendisse vel velit ac est consectetur feugiat nec ac urna. Ut faucibus ex nec dictum fermentum. Morbi aliquet purus at sollicitudin ultrices.",
                        "details": "Ut convallis eleifend orci vel venenatis. Duis vulputate metus in lacus sollicitudin vestibulum. Suspendisse vel velit ac est consectetur feugiat nec ac urna. Ut faucibus ex nec dictum fermentum. Morbi aliquet purus at sollicitudin ultrices. Quisque viverra varius cursus. Praesent sed mauris gravida, pharetra turpis non, gravida eros.",
                        "requires": "2"
                    },
                    "author": self.source,
                    "voting_start_epoch": self.start_epoch,
                    "voting_end_epoch": self.end_epoch,
                    "grace_epoch": self.grace_epoch
                }
            });
        proposal
        }
    }
    fn generate_proposal_json_file(
        proposal_path: &std::path::Path,
        proposal_content: &serde_json::Value,
    ) {
        let intent_writer = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(proposal_path)
            .unwrap();
    
        serde_json::to_writer(intent_writer, proposal_content).unwrap();
    }
}

