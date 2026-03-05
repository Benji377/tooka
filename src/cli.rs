use colored::Colorize;

pub fn show_banner() {
    let banner = r"
████████  ██████   ██████  ██   ██  █████  
   ██    ██    ██ ██    ██ ██  ██  ██   ██ 
   ██    ██    ██ ██    ██ █████   ███████ 
   ██    ██    ██ ██    ██ ██  ██  ██   ██ 
   ██     ██████   ██████  ██   ██ ██   ██ 
                                           
";

    println!("{}", banner.bright_cyan().bold());
    println!(
        "{}",
        "🚀 A fast, rule-based CLI tool for organizing your files".bright_white()
    );
    println!();
    println!(
        "{}",
        "Run `tooka --help` for usage information".bright_yellow()
    );
    println!(
        "{}",
    "Visit https://github.com/benji377/tooka for documentation".bright_blue()
    );
    println!();
}

pub fn success(message: &str) {
    println!("{} {}", "[OK]".green().bold(), message.green());
}

pub fn error(message: &str) {
    eprintln!("{} {}", "[ERR]".red().bold(), message.red());
}

pub fn warning(message: &str) {
    println!("{} {}", "[WARN]".yellow().bold(), message.yellow());
}

pub fn info(message: &str) {
    println!("{} {}", "[INFO]".blue().bold(), message.bright_white());
}

pub fn header(title: &str) {
    println!();
    println!("{}", title.bright_cyan().bold().underline());
    println!();
}

pub fn rule_table_header() {
    println!(
        "{} | {} | {}",
        "Rule ID".bright_cyan().bold(),
        "Name".bright_cyan().bold(),
        "Enabled".bright_cyan().bold()
    );
    println!("{}", "─".repeat(80).bright_black());
}

pub fn rule_table_row(id: &str, name: &str, enabled: bool) {
    let status = if enabled {
        "✓ Enabled".green()
    } else {
        "✗ Disabled".red()
    };

    println!(
        "{:<30} | {:<30} | {}",
        id.bright_white(),
        name.white(),
        status
    );
}

pub fn progress_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-")
}

pub fn show_version() {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("{}", "🚀 Tooka".bright_cyan().bold());
    println!("{} {}", "Version:".bright_white(), version.green().bold());
    println!(
        "{} {}",
        "Repository:".bright_white(),
    "https://github.com/benji377/tooka".blue()
    );
    println!(
        "{} {}",
        "Website:".bright_white(),
        "https://tooka.deno.dev".blue()
    );
    println!();
}
