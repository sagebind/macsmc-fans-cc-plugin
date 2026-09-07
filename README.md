# Apple Silicon plugin for CoolerControl

[![Build](https://github.com/sagebind/macsmc-fans-cc-plugin/workflows/build/badge.svg)](https://github.com/sagebind/macsmc-fans-cc-plugin/actions)

This is a device plugin for [CoolerControl](https://docs.coolercontrol.org) that implements support for controlling system fans on Apple Silicon Mac computers running [Asahi Linux](https://asahilinux.org). After installing this plugin, CoolerControl will be able to apply custom fan curves to the system fans, and CoolerControl can then be used as basically a sort of user-friendly Linux equivalent of [Macs Fan Control](https://crystalidea.com/macs-fan-control).

The plugin expects the [`macsmc-hwmon`](https://docs.kernel.org/hwmon/macsmc-hwmon.html) kernel module to be loaded. Additionally, you must set the `fan_control=1` kernel module option in order to allow userspace to take control of the fans. This can be set using a modprobe config file like this:

```sh
echo "options macsmc_hwmon fan_control=1" | sudo tee /etc/modprobe.d/macsmc-fancontrol.conf
```

Then rebuilding your initramfs.

## :warning: Warning

The Asahi Linux team does **not** recommend enabling userspace control of the system fans, as the Mac firmware will not protect you from burning up your CPU! If you choose inappropriate fan curves in CoolerControl, or if CoolerControl stops working properly for any reason, the system may overheat and cause permanent hardware damage.

## Hardware support

In theory, any device supported by `macsmc-hwmon` should work with this plugin.

## Installation

### Fedora

Prebuilt binaries are published to the [sagebind/coolercontrol-plugins](https://copr.fedorainfracloud.org/coprs/sagebind/coolercontrol-plugins/) COPR repository for Fedora. You can install this plugin from there:

```sh
sudo dnf copr enable sagebind/coolercontrol-plugins
sudo dnf install macsmc-fans-cc-plugin
```

### Manual

Installing the plugin can be done by cloning this repository and running

```sh
make install
```

You will need working Rust protobuf compilers installed for building the plugin.
