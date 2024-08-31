pub mod cell;
pub mod grab_circle;
pub mod magnetar;
pub mod ring;

use color_eyre::eyre::Result;
use magnetar::Magnetar;
use stardust_xr_fusion::{client::Client, node::NodeType, root::RootAspect};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	let (client, event_loop) = Client::connect_with_async_loop()
		.await
		.expect("Unable to connect to server");

	let mut root = Magnetar::new(&client)?;
	root.add_cell();
	let _root_wrapper = client.get_root().alias().wrap(root)?;

	tokio::select! {
		e = tokio::signal::ctrl_c() => e?,
		e = event_loop => e??,
	};
	Ok(())
}
