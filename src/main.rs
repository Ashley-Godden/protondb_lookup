use std::{str::FromStr, io::{self, stdout, Write}};
use scraper::{Html, Selector};

extern crate reqwest;
extern crate scraper;
extern crate serde_json;


// Simple function for clearing the screen
fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    let _ = stdout().flush();
}


// Get if a game is dlc
fn is_game_dlc(app_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let mut url = String::from_str("https://store.steampowered.com/api/appdetails?appids=").unwrap();
    url.push_str(app_id);

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap();
    let res: serde_json::Value = client.get(&url)
        .send()?
        .json()?;

    let mut is_dlc: bool = false;
    let app_type = res[app_id]["data"]["type"].clone();

    if app_type != "game" {
        is_dlc = true;
    }
    Ok(is_dlc)
}


// Get a list of games and app ids from 'store.steampowered.com' using the input of a Game Name
fn get_games(game_name: &String) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    // Create the desired url and add the game_name as a search term
    let mut url: String = String::from_str("https://store.steampowered.com/search/?term=").unwrap();
    url.push_str(game_name);

    // Build to HTTP client using the 'reqwest' crate
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let res = client.get(url) // Get the result from the server and convert it to Text
        .send()?
        .text()?;

    let document = Html::parse_document(res.as_str()); // Parse the output from the server using the Html Parser
    let selector = Selector::parse(r#"a[class="search_result_row ds_collapse_flag "]"#).unwrap(); // Create a selector to be used to find all results

    let mut games_list: Vec<Vec<String>> = Vec::new(); // Create an empty Vector of String Vectors to store the titles and app ids of the results

    // Iterate through all the elements in the document that match our selector
    for element in document.select(&selector) {
        let app_id_wrapped = element.value().attr("data-ds-appid"); // Get the app id from the element
        if app_id_wrapped.is_none() { // Move onto the next element if this element doesn't have an app id associated with it
            continue;
        }
        let app_id = app_id_wrapped.unwrap(); // Unwrap the appid and store it in a &str
        if app_id.contains(",") { // If the appid contains a ',' char then move onto the next element
            continue;
        }

        let title_selector = Selector::parse(r#"span[class="title"]"#).unwrap(); // Create a selector to find the title of the game in the element
        let title_wrapped = element.select(&title_selector).next(); // Find the first instance of the title class in the element and store it

        if title_wrapped.is_none() { // If the title is none then move onto the next element
            continue;
        }

        let title = title_wrapped.unwrap().text().next().unwrap(); // Get the title and store it in a &str

        let is_dlc_wrapped = is_game_dlc(app_id);

        if is_dlc_wrapped.is_err() {
            println!("Couldn't determine if [{}] is DLC", title);
            continue;
        }

        let is_dlc = is_dlc_wrapped.unwrap();

        if is_dlc {
            continue;
        }

        let mut game_info: Vec<String> = Vec::new(); // Create an empty Vector of Strings
        game_info.push(String::from_str(title).unwrap()); // Push the Game Title to the Vector
        game_info.push(String::from_str(app_id).unwrap()); // Push the App ID to the Vector

        games_list.push(game_info.clone()); // Push the parsed game info the the games list
    }

    Ok(games_list) // Return the games info list
}


// Search protondb for the games rating
fn search_protondb(app_id: &String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut url = String::from_str("https://www.protondb.com/api/v1/reports/summaries/").unwrap(); // Create base URL for protondb search
    url.push_str(app_id.as_str()); // Add the appid to the URL
    url.push_str(".json"); // Append .json to the url as we are grabbing a json file

    let client = reqwest::blocking::Client::builder() // Build the HTTP client
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let res = client.get(url) // Get the result from the HTTP request and get the JSON data
        .send()?
        .json()?;

    Ok(res) // Return json data
}


// Main function. APP starts here
fn main() {
    clear_screen();

    let mut game_name = String::new(); // Create a variable to store the desired games name
    println!("Input Game Name:");
    print!("> ");
    let _ = stdout().flush();
    io::stdin().read_line(&mut game_name).expect("Failed to read input"); // Get the game the user wishes to look for

    clear_screen();
    println!("############################################");
    
    let game_name: String = String::from_str(&game_name).unwrap(); // Unwrap the game name after the user has inputted their choice

    let games_list = get_games(&game_name).unwrap(); // Get all games on the steam store that relate to the desired search term

    if games_list.len() == 0 { // If no games are found then exit the application
        println!("No Games Found!!!");
        return;
    }

    let temp_list = &games_list; // Create a temporary list to itterate through

    let mut counter: usize = 0;
    for game in temp_list { // Print a list of all the games that have been found
        println!("[{}]: {}", counter, game[0]);
        counter += 1;
    }

    let mut choice = String::new(); // Create a variable to store the users choice for what games proton info they want
    println!("############################################");
    print!("> ");
    let _ = stdout().flush();
    io::stdin().read_line(&mut choice).expect("Failed to read input"); // Read the users input

    let formatted_choice = choice.replace("\n", "").replace("\r", ""); // Remove line breaks from the users input
    let choice_int_wrapped = formatted_choice.parse::<usize>(); // Type Cast the users choice to usize

    if choice_int_wrapped.is_err() { // Check if failed to Type Cast the users choice to usize
        println!("Choice must be a number!!!");
        return;
    }

    let choice_int: usize = choice_int_wrapped.unwrap(); // unwrap the type casted choice

    if choice_int >= games_list.len() { // Check if the users choice is greater than the total number of choices available
        println!("Invalid choice selected!!!");
        return;
    }

    let game_choice: Vec<String> = games_list[choice_int].clone(); // Get the desired game the user wishes to get information for
    
    let protondb_info_wrapped = search_protondb(&game_choice[1]); // Get the protondb info for the desired game

    if protondb_info_wrapped.is_err() { // Check if getting the protondb info failed
        println!("Failed to retrieve ProtonDB info!!!");
        return;
    }

    let protondb_info = protondb_info_wrapped.unwrap(); // Unwrap the protondb info

    clear_screen();
    println!("###################################"); // Print the protondb info to the terminal
    println!("ProtonDB Info [{}] [{}]:", game_choice[0], game_choice[1]);
    println!("Best Reported Tier: {}", protondb_info["bestReportedTier"]);
    println!("Current Tier: {}", protondb_info["tier"]);
    println!("Trending Tier: {}", protondb_info["trendingTier"]);
    println!("Total User Reviews: {}", protondb_info["total"]);
    println!("ProtonDB Score: {}", protondb_info["score"]);
    println!("Confidence: {}", protondb_info["confidence"]);
}
