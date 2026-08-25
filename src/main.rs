use render::App;
use winit::event_loop::ControlFlow::Poll;
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(Poll);

    let mut app = App::default();
    Ok(event_loop.run_app(&mut app)?)
}
