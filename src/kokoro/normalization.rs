use regex::Regex;

pub fn normalize_for_kokoro(mut s: String) -> String {
    // 1) Unicode punctuation to ASCII
    s = s.replace(['\u{201C}','\u{201D}'], "\"")
         .replace(['\u{2018}','\u{2019}'], "'")
         .replace(['\u{2013}','\u{2014}'], "-")
         .replace('\u{00A0}', " ");

    // 2) Simple acronym splitting: GPU -> G P U (only if all caps length>=2)
    let re_acronym = Regex::new(r"\b([A-Z]{2,})\b").unwrap();
    s = re_acronym.replace_all(&s, |caps: &regex::Captures| {
        caps[1].chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ")
    }).into_owned();

    // 3) CamelCase spacing: HelloWorld -> Hello World
    let re_camel = Regex::new(r"([a-z])([A-Z])").unwrap();
    s = re_camel.replace_all(&s, "$1 $2").into_owned();

    // 4) Basic number expansion (tiny demo; extend as needed)
    // Years 1900-2099
    let re_year = Regex::new(r"\b(19|20)(\d{2})\b").unwrap();
    s = re_year.replace_all(&s, |caps: &regex::Captures| {
        let century = &caps[1];
        let yy = &caps[2];
        let first = if century == "19" { "nineteen" } else { "twenty" };
        format!("{first} {}", two_digit_to_words(yy))
    }).into_owned();

    // Ordinals 1st..4th
    s = Regex::new(r"\b1st\b").unwrap().replace_all(&s, "first").into_owned();
    s = Regex::new(r"\b2nd\b").unwrap().replace_all(&s, "second").into_owned();
    s = Regex::new(r"\b3rd\b").unwrap().replace_all(&s, "third").into_owned();
    s = Regex::new(r"\b4th\b").unwrap().replace_all(&s, "fourth").into_owned();

    // 5) Space around certain symbols (if they exist in vocab)
    for sym in ["-", "/", "+"] {
        s = s.replace(sym, &format!(" {sym} "));
    }

    // 6) Collapse whitespace
    let re_ws = Regex::new(r"\s+").unwrap();
    s = re_ws.replace_all(&s, " ").trim().to_string();

    s
}

fn two_digit_to_words(yy: &str) -> String {
    // ultra-minimal 00..99; extend for your needs
    match yy {
        "00" => "hundred".to_string(),
        "01" => "oh one".to_string(),
        "02" => "oh two".to_string(),
        "10" => "ten".to_string(),
        "11" => "eleven".to_string(),
        "12" => "twelve".to_string(),
        "20" => "twenty".to_string(),
        "21" => "twenty one".to_string(),
        "22" => "twenty two".to_string(),
        _ => yy.chars().map(|c| digit_word(c)).collect::<Vec<_>>().join(" "),
    }
}
fn digit_word(c: char) -> String {
    match c {
        '0' => "zero",
        '1' => "one",
        '2' => "two",
        '3' => "three",
        '4' => "four",
        '5' => "five",
        '6' => "six",
        '7' => "seven",
        '8' => "eight",
        '9' => "nine",
        _ => "",
    }.to_string()
}
