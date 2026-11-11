//! Command line of the desktop harness.

use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    /// Render this screen to a PNG file instead of opening a window.
    pub screenshot: Option<(String, PathBuf)>,
    pub width: u32,
    pub height: u32,
    /// BCP-47 language tag to use instead of the system language.
    pub locale: Option<String>,
    /// Stretch every text by 40% and add accents, to test truncation and font coverage.
    pub pseudo_locale: bool,
    /// Remote control keys to press, comma-separated, before taking the screenshot.
    pub keys: Option<String>,
    /// Show the clock in 12-hour format.
    pub clock_12h: bool,
    /// How many favorites to put in the dock (the first N demo apps) instead of the first-use setup.
    pub favorites: Option<usize>,
    /// Take the screenshot this many ms after the first splash frame (lume leaves and comes back
    /// after the keys), to see the splash and its reveal.
    pub splash: Option<u64>,
}

pub const USAGE: &str = "\
usage: lume-desktop [--size WxH] [--locale TAG] [--pseudo-locale] [--keys LIST] [--favorites N] [--splash MS] [--screenshot SCREEN FILE.png]

  --size WxH               surface size, e.g. 1280x720 (default) or 1920x1080
  --locale TAG             language to show, e.g. pt-BR or en
  --pseudo-locale          lengthen texts by 40% and accent them
  --keys LIST              press keys first, e.g. down,right,right,ok (up down left right ok back home menu)
  --clock-12h              show the clock in 12-hour format
  --favorites N            put the first N demo apps in the dock (0 to 12)
  --splash MS              come back to lume after the keys and shoot MS ms after the first splash
                           frame (no icons load here: the reveal starts at 600)
  --screenshot SCREEN FILE render SCREEN to a PNG file and exit
";

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut options = Options {
        screenshot: None,
        width: 1280,
        height: 720,
        locale: None,
        pseudo_locale: false,
        keys: None,
        clock_12h: false,
        favorites: None,
        splash: None,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "--size" => {
                let size = value("--size")?;
                let (w, h) = size
                    .split_once('x')
                    .ok_or_else(|| format!("invalid size '{size}', use WxH"))?;
                options.width = w
                    .parse()
                    .map_err(|_| format!("invalid width in '{size}'"))?;
                options.height = h
                    .parse()
                    .map_err(|_| format!("invalid height in '{size}'"))?;
                if options.width == 0 || options.height == 0 {
                    return Err(format!("invalid size '{size}'"));
                }
            }
            "--locale" => options.locale = Some(value("--locale")?),
            "--pseudo-locale" => options.pseudo_locale = true,
            "--keys" => options.keys = Some(value("--keys")?),
            "--clock-12h" => options.clock_12h = true,
            "--favorites" => {
                let n = value("--favorites")?;
                let count: usize = n
                    .parse()
                    .map_err(|_| format!("invalid number of favorites '{n}'"))?;
                if count > 12 {
                    return Err(format!("the dock holds at most 12 favorites, not {count}"));
                }
                options.favorites = Some(count);
            }
            "--splash" => {
                let ms = value("--splash")?;
                options.splash = Some(
                    ms.parse()
                        .map_err(|_| format!("invalid splash time '{ms}'"))?,
                );
            }
            "--screenshot" => {
                let screen = value("--screenshot")?;
                let file = value("--screenshot")?;
                options.screenshot = Some((screen, PathBuf::from(file)));
            }
            "-h" | "--help" => return Err(String::new()),
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Options, String> {
        parse(args.iter().map(|a| a.to_string()))
    }

    #[test]
    fn defaults() {
        let o = parse_args(&[]).unwrap();
        assert_eq!(
            (o.width, o.height, o.screenshot, o.locale, o.pseudo_locale),
            (1280, 720, None, None, false)
        );
    }

    #[test]
    fn all_options() {
        let o = parse_args(&[
            "--size",
            "1920x1080",
            "--locale",
            "pt-BR",
            "--pseudo-locale",
            "--screenshot",
            "home",
            "out.png",
        ])
        .unwrap();
        assert_eq!((o.width, o.height), (1920, 1080));
        assert_eq!(o.locale.as_deref(), Some("pt-BR"));
        assert!(o.pseudo_locale);
        assert_eq!(
            o.screenshot,
            Some(("home".to_string(), PathBuf::from("out.png")))
        );
    }

    #[test]
    fn favorites_option() {
        assert_eq!(
            parse_args(&["--favorites", "3"]).unwrap().favorites,
            Some(3)
        );
        assert_eq!(parse_args(&[]).unwrap().favorites, None);
    }

    #[test]
    fn bad_input_is_rejected() {
        assert!(parse_args(&["--size", "big"]).is_err());
        assert!(parse_args(&["--size", "0x720"]).is_err());
        assert!(parse_args(&["--locale"]).is_err());
        assert!(parse_args(&["--screenshot", "home"]).is_err());
        assert!(parse_args(&["--nope"]).is_err());
        assert!(parse_args(&["--favorites", "13"]).is_err());
        assert!(parse_args(&["--favorites", "many"]).is_err());
        assert!(parse_args(&["--splash", "soon"]).is_err());
        assert_eq!(parse_args(&["--splash", "725"]).unwrap().splash, Some(725));
    }
}
