# magnetar
Workspaces client for Stardust XR. Creates a moveable cylindrical area within which all other clients and windows will move as a group
> [!IMPORTANT]  
> Requires the [Stardust XR Server](https://github.com/StardustXR/server) to be running.

If you installed the Stardust XR server via:  
```note
sudo dnf group install stardust-xr
```
Or if you installed via the [installation script](https://github.com/cyberneticmelon/usefulscripts/blob/main/stardustxr_setup.sh), magnetar comes pre-installed

# How to Use
Run with `magnetar`, and then in flatscreen mode use `right click` to drag it around, and in XR grab with the hand tracking grab

## Manual Installation
Clone the repository and after the server is running:
```sh
cargo run
```
