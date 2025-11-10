mod parser;
mod cfg;
mod renderer;

pub use parser::{parse_c_code, Function, Statement};
pub use cfg::{build_cfg, ControlFlowGraph};
pub use renderer::render_mermaid;
