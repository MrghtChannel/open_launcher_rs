# Open Launcher

Open Launcher is a package to install and launch modded and vanilla Minecraft instances totally automatically with Rust.

## Note about Java

Java is required to run the game. This package does not download a JRE: use the Java installation already available on the computer. Set `OPEN_LAUNCHER_JAVA_PATH` to the absolute path of the executable (for example, `C:\\Program Files\\Java\\jdk-21\\bin\\java.exe`). If it is not set, the launcher runs `java` from the system `PATH`.

The Java executable may be located anywhere. If your launcher lets a user choose it in the UI, pass the selected path to `set_java_executable` before launching:

```rust
launcher.set_java_executable(r"D:\\My Launcher\\Runtime\\Java 21\\bin\\java.exe");
```

## Example usage

```rust
use open_launcher::{auth, version, Launcher};
use std::{env, path};

#[tokio::main]
async fn main() {
    let java_path = env::var("OPEN_LAUNCHER_JAVA_PATH").unwrap_or_else(|_| "java".to_string());
    let mut launcher = Launcher::new(
        path::Path::new(env::home_dir().unwrap().as_path())
            .join(".open_launcher")
            .to_str()
            .unwrap(),
        &java_path,
        version::Version {
            minecraft_version: "1.20.2".to_string(),
            loader: None,
            loader_version: None,
        },
    )
    .await;

    launcher.auth(auth::OfflineAuth::new("Player"));
    launcher.custom_resolution(1280, 720);
    // launcher.fullscreen(true);
    // launcher.quick_play("multiplayer", "hypixel.net");

    let mut progress = launcher.on_progress();
    tokio::spawn(async move {
        loop {
            match progress.recv().await {
                Ok(progress) => {
                    println!(
                        "Progress: {} {}/{} ({}%)",
                        progress.task,
                        progress.current,
                        progress.total,
                        match progress.total {
                            0 => 0,
                            _ => (progress.current as f64 / progress.total as f64 * 100.0 * 100.0)
                                .round() as u64,
                        } as f32
                            / 100.0
                    );
                }
                Err(_) => {
                    println!("Progress channel closed");
                    break;
                }
            }
        }
    });

    match launcher.install_version().await {
        Ok(_) => println!("Version installed successfully."),
        Err(e) => println!("An error occurred while installing the version: {}", e),
    };

    match launcher.install_assets().await {
        Ok(_) => println!("Assets installed successfully."),
        Err(e) => println!("An error occurred while installing the assets: {}", e),
    };

    match launcher.install_libraries().await {
        Ok(_) => println!("Libraries installed successfully."),
        Err(e) => println!("An error occurred while installing the libraries: {}", e),
    };

    let mut process = match launcher.launch() {
        Ok(p) => p,
        Err(e) => {
            println!("An error occurred while launching the game: {}", e);
            std::process::exit(1);
        }
    };

    let _ = process.wait();

    println!("Game closed.");
}
```

More examples can be found in the [examples](./examples/) directory.

## Documentation

The documentation can be found [here](https://docs.rs/open_launcher).

## License

This project is licensed under the MIT License - see the [LICENSE.md](./LICENSE.md) file for details.
