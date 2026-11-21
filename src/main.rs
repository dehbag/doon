use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// ANSI Colors
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[91m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const BLUE: &str = "\x1b[94m";
const MAGENTA: &str = "\x1b[95m";
const CYAN: &str = "\x1b[96m";
const WHITE: &str = "\x1b[97m";

#[derive(Parser, Debug)]
#[command(name = "doon", about = "🚀 Download/clone all GitHub repositories with style!")]
struct Args {
    /// GitHub page URL or org/user name (e.g. https://github.com/nextcloud)
    #[arg(index = 1)]
    target: String,

    /// Destination directory (defaults to /Users/mohsen/git)
    #[arg(short, long, default_value = "/Users/mohsen/git")]
    destination: PathBuf,

    /// Optional GitHub Personal Access Token
    #[arg(short = 't', long)]
    token: Option<String>,

    /// Automatically confirm and start download without asking
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Debug, Deserialize)]
struct GithubEntity {
    #[serde(rename = "type")]
    entity_type: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Repository {
    name: String,
    clone_url: String,
    archived: bool,
    #[serde(default)]
    size: u64, // Size in KB
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let owner = extract_owner(&args.target)?;
    let dest_path = args.destination.clone();

    println!();
    println!("{BOLD}{MAGENTA}✨ 🚀 DOON — GITHUB DOWNLOADER 🚀 ✨{RESET}");
    println!("{DIM}{}{RESET}", "─".repeat(58));
    println!("{BOLD}🎯 Target:{RESET}      {WHITE}{}{RESET}", args.target);
    println!("{BOLD}📂 Destination:{RESET} {GREEN}{}{RESET}", dest_path.display());
    println!("{DIM}{}{RESET}", "─".repeat(58));

    let token = args.token.or_else(|| env::var("GITHUB_TOKEN").ok());
    let client = build_client(token.as_deref())?;

    // Detect User vs Org
    let entity_url = format!("https://api.github.com/users/{owner}");
    let resp = client.get(&entity_url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        eprintln!("{RED}❌ Error: GitHub user/organization '{owner}' not found.{RESET}");
        std::process::exit(1);
    } else if !resp.status().is_success() {
        let err_text = resp.text().await?;
        eprintln!("{RED}❌ GitHub API error: {err_text}{RESET}");
        std::process::exit(1);
    }

    let entity: GithubEntity = resp.json().await?;
    let is_org = entity.entity_type.eq_ignore_ascii_case("Organization");
    let emoji = if is_org { "🏢" } else { "👤" };
    println!("{BOLD}{emoji} Account:{RESET}     {CYAN}{owner}{RESET} {DIM}({}){RESET}", entity.entity_type);

    println!("{YELLOW}🔍 Scanning repositories on GitHub...{RESET}");
    let repos = fetch_all_repos(&client, &owner, is_org).await?;
    let total = repos.len();

    if total == 0 {
        println!("\n{YELLOW}⚠️  No public repositories found for '{owner}'.{RESET}");
        return Ok(());
    }

    tokio::fs::create_dir_all(&dest_path).await?;

    // Calculate sizes
    let mut total_size_kb: u64 = 0;
    let mut new_count: usize = 0;
    let mut new_size_kb: u64 = 0;
    let mut existing_count: usize = 0;
    let mut existing_size_kb: u64 = 0;

    for r in &repos {
        total_size_kb += r.size;
        let repo_dir = dest_path.join(&r.name);
        if repo_dir.exists() && repo_dir.join(".git").exists() {
            existing_count += 1;
            existing_size_kb += r.size;
        } else {
            new_count += 1;
            new_size_kb += r.size;
        }
    }

    // Summary Card
    println!();
    println!("{CYAN}┌{}┐{RESET}", "─".repeat(56));
    println!("{CYAN}│{RESET}  {BOLD}📊 DOWNLOAD SUMMARY{RESET}                                {CYAN}│{RESET}");
    println!("{CYAN}├{}┤{RESET}", "─".repeat(56));
    println!("{CYAN}│{RESET}  🎯 Account:               {BOLD}{owner:<29}{RESET} {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}  📦 Total Repositories:    {BOLD}{total:<29}{RESET} {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}  ⬇️  To Clone (new):        {GREEN}{:<29}{RESET} {CYAN}│{RESET}", format!("{} ({})", new_count, format_size(new_size_kb)));
    println!("{CYAN}│{RESET}  🔄 Already Present (pull):{BLUE}{:<29}{RESET} {CYAN}│{RESET}", format!("{} ({})", existing_count, format_size(existing_size_kb)));
    println!("{CYAN}├{}┤{RESET}", "─".repeat(56));
    println!("{CYAN}│{RESET}  💾 {BOLD}Total Estimated Size:  {YELLOW}{:<29}{RESET} {CYAN}│{RESET}", format!("~{}", format_size(total_size_kb)));
    println!("{CYAN}└{}┘{RESET}", "─".repeat(56));

    // Confirmation
    if !args.yes {
        print!("\n{BOLD}{YELLOW}❓ Do you want to proceed with downloading? [y/N]: {RESET}");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        if choice != "y" && choice != "yes" {
            println!("{RED}🛑 Download cancelled by user. Goodbye! 👋{RESET}");
            return Ok(());
        }
    }

    println!("\n{BOLD}{GREEN}🚀 Starting download process...{RESET}\n");

    for (idx, repo) in repos.iter().enumerate() {
        let repo_dir = dest_path.join(&repo.name);
        let archived_tag = if repo.archived { format!(" {DIM}[📦 Archived]{RESET}") } else { "".to_string() };
        println!(
            "{BOLD}[{}/{total}]{RESET} 📦 {WHITE}{BOLD}{}{RESET} {CYAN}({}){RESET}{archived_tag}",
            idx + 1,
            repo.name,
            format_size(repo.size)
        );

        sync_repository(repo, &repo_dir)?;
        println!();
    }

    println!("{GREEN}{}{RESET}", "═".repeat(58));
    println!("🎉 {BOLD}{GREEN}All {total} repositories successfully synced to {}!{RESET} ✨", dest_path.display());
    println!("{GREEN}{}{RESET}\n", "═".repeat(58));

    Ok(())
}

fn format_size(kb: u64) -> String {
    if kb < 1024 {
        format!("{kb} KB")
    } else if kb < 1024 * 1024 {
        format!("{:.2} MB", kb as f64 / 1024.0)
    } else {
        format!("{:.2} GB", kb as f64 / (1024.0 * 1024.0))
    }
}

fn extract_owner(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let trimmed = input.trim().trim_end_matches('/');
    let target = if let Some(stripped) = trimmed.strip_prefix("https://github.com/") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("http://github.com/") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("github.com/") {
        stripped
    } else {
        trimmed
    };

    let owner = target.split('/').next().unwrap_or("").trim();
    if owner.is_empty() {
        return Err("Could not extract username or organization from target".into());
    }
    Ok(owner.to_string())
}

fn build_client(token: Option<&str>) -> Result<reqwest::Client, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("doon-cli"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github+json"));

    if let Some(tok) = token {
        let auth_val = format!("Bearer {tok}");
        let mut header_val = HeaderValue::from_str(&auth_val)?;
        header_val.set_sensitive(true);
        headers.insert(AUTHORIZATION, header_val);
    }

    Ok(reqwest::Client::builder().default_headers(headers).build()?)
}

async fn fetch_all_repos(
    client: &reqwest::Client,
    owner: &str,
    is_org: bool,
) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
    let mut all_repos = Vec::new();
    let mut page = 1;
    let per_page = 100;

    loop {
        let url = if is_org {
            format!("https://api.github.com/orgs/{owner}/repos?per_page={per_page}&page={page}&type=public")
        } else {
            format!("https://api.github.com/users/{owner}/repos?per_page={per_page}&page={page}&type=owner")
        };

        let response = client.get(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format!("API error on page {page} ({status}): {body}").into());
        }

        let repos_on_page: Vec<Repository> = response.json().await?;
        if repos_on_page.is_empty() {
            break;
        }

        let count = repos_on_page.len();
        all_repos.extend(repos_on_page);
        println!("   {DIM}↳ Page {page}: fetched {count} repos (total so far: {}){RESET}", all_repos.len());

        if count < per_page {
            break;
        }
        page += 1;
    }

    Ok(all_repos)
}

fn sync_repository(repo: &Repository, repo_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if repo_dir.exists() && repo_dir.join(".git").exists() {
        println!("   {BLUE}🔄 Local folder exists. Running 'git pull'...{RESET}");
        let _ = Command::new("git")
            .current_dir(repo_dir)
            .args(["pull", "--ff-only"])
            .status();
    } else {
        println!("   {CYAN}⬇️  Cloning {}...{RESET}", repo.clone_url);
        let _ = Command::new("git")
            .args(["clone", "--quiet", &repo.clone_url, repo_dir.to_str().unwrap()])
            .status();
    }
    println!("   {GREEN}✨ Done!{RESET}");
    Ok(())
}
