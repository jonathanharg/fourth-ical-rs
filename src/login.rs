use crate::shift_api_url;
use fancy_regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::env;

fn url_regex(regex: &str, doc: &str) -> String {
    let regex = Regex::new(regex).unwrap();
    regex
        .find(doc)
        .unwrap()
        .unwrap()
        .as_str()
        .replace("&amp;", "&")
}

fn parse_attr<'a>(text: &'a str, selector: &'a str, attr: &'a str) -> String {
    let parsed_html = Html::parse_document(text);
    let selector = Selector::parse(selector).expect("Selector is not valid");
    parsed_html
        .select(&selector)
        .next()
        .expect("Cannot find selector")
        .value()
        .attr(attr)
        .expect("Cannot find attribute")
        .to_string()
}

fn parse_inner<'a>(text: &'a str, selector: &'a str) -> String {
    let parsed_html = Html::parse_document(text);
    let selector = Selector::parse(selector).expect("Selector is not valid");
    parsed_html
        .select(&selector)
        .next()
        .expect("Cannot find selector")
        .inner_html()
}
async fn get_text(client: &Client, url: &str) -> String {
    client
        .get(url)
        .send()
        .await
        .expect("GET request expected response, but there was none.")
        .text()
        .await
        .expect("GET request expected a text response.")
}
pub async fn get_cookie(client: &Client) {
    println!("Attempting to get login cookie!");
    // Phase 0 GET FMPLogin
    let user = env::var("LOGIN")
        .expect("No LOGIN environemnt variable set. Please set this to your Fourth email login.");
    let password = env::var("PASSWORD").expect(
        "No PASSWORD environment variable set. Please set this to your Fourth login password.",
    );

    let api = shift_api_url();
    println!("Making request to API.");
    let get_fmplogin = client
        .get(&api)
        .send()
        .await
        .expect("GET request expected response, but there was none.");
    println!("Recieved API response.");

    if get_fmplogin.url().as_str() == api {
        println!("Cookie already exists and is valid.");
        return;
    }

    let get_fmplogin = get_fmplogin
        .text()
        .await
        .expect("GET request expected a text response.");

    let post_fmp_url = parse_attr(&get_fmplogin, "form", "action");
    println!("Parsed API response.");
    let viewstate = parse_attr(
        &get_fmplogin,
        "input[id=\"com.salesforce.visualforce.ViewState\"]",
        "value",
    );
    let viewstate_version = parse_attr(
        &get_fmplogin,
        "input[id=\"com.salesforce.visualforce.ViewStateVersion\"]",
        "value",
    );
    let viewstate_mac = parse_attr(
        &get_fmplogin,
        "input[id=\"com.salesforce.visualforce.ViewStateMAC\"]",
        "value",
    );

    // Phase 1 POST FMPlogin

    let fmplogin_form = [
        ("j_id0:j_id2:j_id15", "j_id0:j_id2:j_id15"),
        ("j_id0:j_id2:j_id15:username", &user),
        ("j_id0:j_id2:j_id15:j_id24", &password),
        ("j_id0:j_id2:j_id15:submit", "Sign+In"),
        ("com.salesforce.visualforce.ViewState", &viewstate),
        (
            "com.salesforce.visualforce.ViewStateVersion",
            &viewstate_version,
        ),
        ("com.salesforce.visualforce.ViewStateMAC", &viewstate_mac),
    ];

    println!("Sending POST request to FMPLogin");
    let post_fmplogin = client
        .post(post_fmp_url)
        .form(&fmplogin_form)
        .send()
        .await
        .expect("POST FMPlogin response expected")
        .text()
        .await
        .expect("POST FMPlogin response has text");
    println!("POST request to FMPLogin sent");
    let fmp_script = parse_inner(&post_fmplogin, "script");
    let frontdoor_url = url_regex(r"(?<=window\.location\.href =').*(?=\')", &fmp_script);
    println!("Parsed frontdoor url");

    // Phase 2 Front Door

    println!("Sending request to frontdor");
    let frontdoor = get_text(client, &frontdoor_url).await;
    println!("Recieved response from frontdoor");
    let login_script = parse_inner(&frontdoor, "script");

    let login_url = url_regex(r#"(?<=window\.location\.href=").*(?=\"})"#, &login_script);
    let login_url = format!("https://secure.fourth.com{}", login_url);
    println!("Parsed frontdoor response");

    // Phase 3 idp login
    println!("Sending request to login");
    let login = get_text(client, &login_url).await;
    println!("Recieved response from login");

    let saml_url = parse_attr(&login, "form", "action");
    let relay_state = parse_attr(&login, r#"input[name="RelayState"]"#, "value");
    let saml_response = parse_attr(&login, r#"input[name="SAMLResponse"]"#, "value");
    println!("Parsed login response");

    // Phase 4 SAML
    println!("Sending POST request to SAML");
    let saml_form = [
        ("RelayState", &relay_state),
        ("SAMLResponse", &saml_response),
    ];
    client
        .post(&saml_url)
        .form(&saml_form)
        .send()
        .await
        .expect("SAML response");
    println!("Cookie recieved!")
    // l7auth_prod cookie has now been set on client.
}

