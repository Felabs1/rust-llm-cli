mod client;
mod commands;
mod config;
mod models;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = config::api_key()?;
    let prompt = commands::parse_commands()?;
    let reply = client::ask(&api_key, &prompt)?;

    println!("{}", reply.choices[0].message.content);
    println!("{}", reply.choices[0].finish_reason);
    println!("{}", reply.id);
    println!("{}", reply.model);

    Ok(())
}
