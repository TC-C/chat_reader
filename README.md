<p align="center"><i>If it compiles, it's good; if it boots up, it's perfect - Linus Torvalds</i></p>

# Read and search through the chat from any VOD on Twitch or AfreecaTV
Currently, Twitch and AfreecaTV are the only supported platforms.

In addition, this tool retrieves direct M3U8 links to Twitch Vods and can search through clips on Twitch Channel

# Build instructions 
<img src="https://techworm.page/wp-content/uploads/2019/05/download-17.png" width=275 height=150></img>

(If you haven't installed already)
Install Rust; You can use [Rustup](https://www.rust-lang.org/tools/install)

Clone the project to some directory:
`git clone https://github.com/TC-C/vod_search.git`

Next, we can build with Cargo:
`cd vod_search && cargo build --release`

You can then run the binary (on *nix OS) with
`./target/release/chat_reader`

(On Windows? The .exe will be located in the same folder)

