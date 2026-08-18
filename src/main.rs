use std::io::{self, Write};    // Inputs
use std::process::Command;     // Commands
use std::process;              // Process handling
use std::fs;                   // Read file
use std::path::Path;           // Check existing checkout

const WAYWALL_REPO_URL: &str = "https://github.com/tesselslate/waywall";

fn main() {
    // ==== Detect distro =====================================================
    let _distro = fs::read_to_string("/etc/os-release").unwrap();
    let distro: &str = _distro
        .lines()
        .find(|l| l.starts_with("ID_LIKE="))
        .or_else(|| _distro.lines().find(|l| l.starts_with("ID=")))
        .and_then(|l| l.split('=').nth(1))
        .and_then(|l| l.split(' ').last())
        .map(|l| l.trim_matches('"'))
        .unwrap_or("unknown");

    // ==== Options (defaults) ================================================
    let installation_type: i32;                     // 0: cancel    1: default  2: custom
    let mut waywall_install: i32 = 1;               // 0: cancel    1: stable   2: latest   3: skip
    let mut instances: Vec<&str> = Vec::new();      // Vector of instance names
    let mut is_nvidia: bool = false;                // For the environment variable
    let mut is_internal_gpu: bool = false;          // If needed to not check "Use Discrete GPU"
    let mut is_latest_version: bool = false;        // For all the 26.1 tech
    let mut use_generic_config: bool = true;        // Clones generic config to ~/.config/waywall
    let waywall_release_tag: &str = "0.2026.06.13"; // Waywall release tag in github releases
    let _user: String = String::from_utf8(Command::new("logname").output().expect("err").stdout)
        .unwrap();        // Get user because of some wierd behaviour
    let user: &str = _user.trim();

    // ==== Prompts ===========================================================

    // == installation_type ===================================================
    header();
    let input = ask(r#"
======== What installation type do you want? ========

1) Default (recommended)

2) Custom 

Press Enter to cancel installation
    "#);

    installation_type = to_int(&input, 1, 2);

    if installation_type == 0 {
        // Cancel Install
        println!("Canceling Installation");
        return;
    }
    else if installation_type == 2 {
        println!("Rest of Prompts");


        // == waywall_install =================================================
        header();
        let input = ask(r#"
======= What version of Waywall do you want? ========

1) Stable (recommended)

2) Latest (built from source)

3) Skip (if waywall's already installed)

Press Enter to cancel installation
    "#);
        waywall_install = to_int(&input, 1, 3);
        if waywall_install == 0 {
            // Cancel Install
            println!("Canceling Installation");
            return;
        }

        // == instances =======================================================
        let mut instance_options: Vec<&str> = Vec::new(); // Vector of existing instances

        // ADD INSTANCES TEMP
        instance_options.push("RSG");
        instance_options.push("SSG");
        instance_options.push("Ranked");

        let mut instances_prompt: String = String::from("\n====== Which instances do you want to set up? =======\n         type each instance number (eg. 123)\n\n0) None\n\n"); 

        for (instance_number, instance_name) in instance_options.iter().enumerate() {
            instances_prompt.push_str(&format!("{}) {}\n\n", instance_number + 1, instance_name));
        }

        header();
        let input = ask(&instances_prompt);

        if to_int(&input, 0, 0) != 0 {
            for digit in input.chars().filter_map(|c| c.to_digit(10)) {
                let index = digit as usize;

                if index > 0 && index <= instance_options.len() {
                    let choice = instance_options[index - 1];
                    instances.push(choice);
                }
            }
        }


        // == is_nvidia =======================================================
        header();
        let input = ask(r#"
============ Are you using an Nvidia GPU? ===========

1) Yes

2) No
    "#);
        let int_input = to_int(&input, 1, 2);
        is_nvidia = int_input == 1;


        // == is_internal_gpu =================================================
        header();
        let input = ask(r#"
=========== Are you using an internal GPU? ==========

1) Yes

2) No
    "#);
        let int_input = to_int(&input, 1, 2);
        is_internal_gpu = int_input == 1;


        // == is_latest_version ===============================================
        header();
        let input = ask(r#"
=== Are you using Waywall on the latest version? ====

1) Yes

2) No
    "#);
        let int_input = to_int(&input, 1, 2);
        is_latest_version = int_input == 1;


        // == use_generic_config ==============================================
        header();
        let input = ask(r#"
===== Do you wish to install a Generic Config? ======

1) Yes

2) No
    "#);
        let int_input = to_int(&input, 1, 2);
        use_generic_config = int_input == 1;

    }

    // == System Update
    header();
        let input = ask(r#"
====== Update system packages before building? ======

1) Yes

2) No
    "#);

    let update_packages = to_int(&input, 1, 2);


    // ==== Confirmation ======================================================
    header();
    println!("");
    println!("Confirm Options:");
    println!("    Waywall Version:          {}", if waywall_install == 1 {"Stable (prebuilt package)"} else if waywall_install == 2 {"Latest (git)"} else {"None"});
    println!("    Instances to Setup:       {}", instances.join(", "));
    println!("    Nvidia GPU:               {}", if is_nvidia {"True"} else {"False"});
    println!("    Internal GPU:             {}", if is_internal_gpu {"True"} else {"False"});
    println!("    Latest Version:           {}", if is_latest_version {"True"} else {"False"});
    println!("    Install Generic Config:   {}", if use_generic_config {"True"} else {"False"});

    
    // ==== Installation ======================================================

    // === Waywall installation
    waywall(waywall_install, distro, waywall_release_tag, user, update_packages);

    // == Install Generic Config
    if use_generic_config {
        install_generic(user);
    }
}

fn waywall(itype: i32, distro: &str, waywall_tag: &str, user: &str, update_packages: i32) {

    if update_packages == 1 {
        update_system_packages(distro);
    }

    // Install waywall
    if itype == 1 {
        // Download the waywall package
        match distro {
            "arch" => run_command(&format!("curl -fsSL {}/releases/download/{}/waywall-0.5-1-x86_64.pkg.tar.zst -o /tmp/waywall.pkg.tar.zst", WAYWALL_REPO_URL, waywall_tag)),
            "fedora" => run_command(&format!("curl -fsSL {}/releases/download/{}/waywall-0.5-1.fc42.x86_64.rpm -o /tmp/waywall.rpm", WAYWALL_REPO_URL, waywall_tag)),
            "debian" => run_command(&format!("curl -fsSL {}/releases/download/{}/waywall_0.5-1_amd64.deb -o /tmp/waywall.deb", WAYWALL_REPO_URL, waywall_tag)),
            _ => println!("Unknown distro type found: {}", distro),
        }
        // Install the waywall package
        match distro {
            "arch" => run_command("sudo pacman -U /tmp/waywall.pkg.tar.zst"),
            "fedora" => run_command("sudo dnf localinstall /tmp/waywall.rpm"),
            "debian" => run_command("sudo apt install -y /tmp/waywall.deb"),
            _ => {
                println!("Unknown distro type found: {}", distro);
                process::exit(1);
            },
        }
    }
    else if itype == 2 {
        // Build from source
        build_from_source(distro, user);
    }
}

fn build_from_source(distro: &str, user: &str) {
    // Install waywall-working-git on Arch if yay exists (future: check if they're using paru instead?)
    if distro == "arch" && command_exists("yay") {
        println!("yay package manager detected, installing/updating waywall-working-git from the AUR");
        run_command("yay -S --needed --noconfirm waywall-working-git");
        return;
    }

    // clone + install otherwise
    let waywall_dir = format!("/home/{}/waywall", user);
    install_build_deps(distro);
    clone_waywall_helper(&waywall_dir);
    run_command(&format!("cd {} && make", waywall_dir));
    run_command(&format!("chown -R {}:{} {}", user, user, waywall_dir));
}

fn install_build_deps(distro: &str) {
    println!("Installing build dependencies for {}", distro);
    match distro {
        "arch" => run_command(
            "sudo pacman -S --needed --noconfirm base-devel git meson ninja \
             libegl libgles luajit libspng wayland wayland-protocols \
             libxcb libxkbcommon xorg-xwayland",
        ),
        "fedora" => run_command(
            "sudo dnf install -y gcc make cmake meson ninja-build pkgconf-pkg-config git \
             wayland-devel wayland-protocols-devel mesa-libEGL-devel mesa-libGLES-devel \
             luajit-devel libspng-devel libxkbcommon-devel libxcb-devel \
             xorg-x11-server-Xwayland-devel",
        ),
        "debian" => {
            run_command("apt update");
            run_command(
                "sudo apt install -y --no-install-recommends build-essential git meson \
                 ninja-build pkg-config cmake wayland-protocols libwayland-dev \
                 libegl-dev libgles-dev libspng-dev libluajit-5.1-dev libxkbcommon-dev \
                 libxcb1-dev libxcb-composite0-dev libxcb-res0-dev libxcb-xtest0-dev xwayland",
            )
        }
        _ => {
            println!("Unknown distro type found: {}", distro);
            process::exit(1);
        }
    }
}

fn clone_waywall_helper(waywall_dir: &str) {
    if Path::new(&format!("{}/.git", waywall_dir)).exists() {
        println!(
            "Existing waywall checkout found at {}, pulling latest",
            waywall_dir
        );
        run_command(&format!("git -C {} fetch", waywall_dir));
        run_command(&format!("git -C {} pull", waywall_dir));
    } else {
        println!("Cloning waywall into {}", waywall_dir);
        run_command(&format!("git clone {} {}", WAYWALL_REPO_URL, waywall_dir));
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {} >/dev/null 2>&1", cmd))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn install_generic(user: &str) {
    // Download generic
    println!("Downloading Gore's generic config");
    run_command(&format!("[ -d /home/{}/.config/waywall ] && mv /home/{}/.config/waywall /home/{}/.config/waywall.bkp >/dev/null 2>&1 || true", user, user, user)); // Check for existing configuration and incase of it existing move it to a backup
    run_command(&format!("git clone https://github.com/arjuncgore/waywall_generic_config.git /home/{}/.config/waywall", user)); // Download it
    println!("Generic config downloaded!");
}

fn update_system_packages(distro: &str) {
    match distro {
        "arch" =>   run_command("sudo pacman -Syu"),
        "fedora" => run_command("sudo dnf upgrade -y"),
        "debian" => run_command("sudo apt update && sudo apt full-upgrade -y"),
        _ => println!("Unknown distro type found: {}", distro),
    }
}

fn header() {
    clearscreen::clear().expect("Failed to clear screen");
    println!(r#"=====================================================
                Waywall Installer CLI
                       by Gore
====================================================="#);
}

fn ask(prompt: &str) -> String {
    println!("{}", prompt);

    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Blank");

    input.trim().to_string()
}

fn to_int(input: &str, min: i32, max: i32) -> i32{
    // Convert to integer
    let int_input: i32 = match input.trim().parse() {
        Ok(num) => {
            if (min == 0) && (max == 0) {
                num
            }
            else if (num >= min) && (num <= max) {
                num
            }
            else {
                0
            }
        }
        Err(_) => {
            0
        }
    };
    int_input
}

fn run_command(cmd: &str) {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .expect("Failed to execute command");

    if !status.success() {
        eprintln!("Command failed with status: {}", status);
    }
}
