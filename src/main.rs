pub mod cell;
pub mod magnetar;
pub mod ring;

use magnetar::Magnetar;
use stardust_xr_molecules::fusion::client::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	let (client, event_loop) = Client::connect_with_async_loop()
		.await
		.expect("Unable to connect to server");

	let mut root = Magnetar::new(&client);
	root.add_cell();
	let _root_wrapper = client.wrap_root(root);

	tokio::select! {
		e = tokio::signal::ctrl_c() => e.unwrap(),
		e = event_loop => e.unwrap().unwrap(),
	}
}