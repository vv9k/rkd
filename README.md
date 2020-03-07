# rkd 
Rusty keybinding daemon for linux. ⌨️ 🦀


## USAGE
To use `rkd`:
 - add your user to group input by `sudo usermod -G input -a $your_username`
 - clone this repository with `git clone https://github.com/wojciechkepka/rkd`
 - build with `cargo build --release`
 - copy to `sudo cp target/release/rkd /usr/bin/`
 - run with `rkd $path_to_conf_file`
 - To debug and see some output use `RUST_LOG=trace /usr/bin/rkd $path_to_conf_file`
---
## CONFIG
the config file has a syntax very similar to `sxhkd`:
```
super + 1
	bspc desktop -f ^1
super + 2
	bspc desktop -f ^2
super + 3
	bspc desktop -f ^3
super + q
	bspc node -c
# You can do either this
super + Q
	bspc node -k
# or this
super + shift + q
	bspc node -k

```
The first key has to be one of `super`|`shift`|`alt`|`ctrl`. The modifier key can be followed by any amount of other mod keys but to actually execute the keybinding on of `[0-9a-z,./;'\\[\]]` has to be pressed.

---
## LICENSE
[**MIT**](https://github.com/wojciechkepka/rkd/blob/master/LICENSE)

