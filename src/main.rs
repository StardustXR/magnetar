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

	let root_wrapper = client.wrap_root(Magnetar::new(&client));
	root_wrapper.lock_inner().add_cell();

	tokio::select! {
		e = event_loop => e.unwrap(),
		e = tokio::signal::ctrl_c() => e.unwrap(),
	}
}
