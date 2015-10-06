use server::game::initial_state::InitialState;
use testing::process::Process;
use testing::util::random_port;


pub struct Server {
    port    : u16,
    _process: Process,
}

impl Server {
    pub fn start(_initial_state: InitialState) -> Server {
        // TODO: Configure server process to use initial state

        let port = random_port(40000, 50000);

        let mut process = Process::start(
            "vndf-server",
            &[
                format!("--port={}"          , port).as_ref(),
                format!("--client-timeout={}", 0.1 ).as_ref(),
                format!("--sleep-duration={}", 5   ).as_ref(),
            ]
        );
        process.read_stderr_line(); // Make sure it's ready

        Server {
            port    : port,
            _process: process,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
