/// Test data for YouTube player solvers
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TestStep {
    pub input: &'static str,
    pub expected: &'static str,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub player: &'static str,
    pub variants: Option<&'static [&'static str]>,
    pub n: &'static [TestStep],
    pub sig: &'static [TestStep],
}

/// All player variants
pub const ALL_VARIANTS: &[&str] = &[
    "main", "tcc", "tce", "es5", "es6", "tv", "tv_es6", "phone", "tablet",
];

/// Variants without tce (for players where tce causes exceptions)
pub const VARIANTS_NO_TCE: &[&str] = &[
    "main", "tcc", "es5", "es6", "tv", "tv_es6", "phone", "tablet",
];

/// Player URL paths for each variant
pub fn get_player_paths() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("main", "player_ias.vflset/en_US/base.js");
    map.insert("tcc", "player_ias_tcc.vflset/en_US/base.js");
    map.insert("tce", "player_ias_tce.vflset/en_US/base.js");
    map.insert("es5", "player_es5.vflset/en_US/base.js");
    map.insert("es6", "player_es6.vflset/en_US/base.js");
    map.insert("tv", "tv-player-ias.vflset/tv-player-ias.js");
    map.insert("tv_es6", "tv-player-es6.vflset/tv-player-es6.js");
    map.insert("phone", "player-plasma-ias-phone-en_US.vflset/base.js");
    map.insert("tablet", "player-plasma-ias-tablet-en_US.vflset/base.js");
    map
}

/// Get cache path for a player file
pub fn get_cache_path(player: &str, variant: &str) -> String {
    format!("players/{}-{}", player, variant)
}

/// All test cases
pub const TEST_CASES: &[TestCase] = &[
    TestCase {
        player: "3d3ba064",
        variants: None,
        n: &[
            TestStep { input: "ZdZIqFPQK-Ty8wId", expected: "qmtUsIz04xxiNW" },
            TestStep { input: "4GMrWHyKI5cEvhDO", expected: "N9gmEX7YhKTSmw" },
        ],
        sig: &[
            TestStep {
                input: "gN7a-hudCuAuPH6fByOk1_GNXN0yNMHShjZXS2VOgsEItAJz0tipeavEOmNdYN-wUtcEqD3bCXjc0iyKfAyZxCBGgIARwsSdQfJ2CJtt",
                expected: "ttJC2JfQdSswRAIgGBCxZyAfKyi0cjXCb3gqEctUw-NYdNmOEvaepit0zJAtIEsgOV2SXZjhSHMNy0NXNG_1kNyBf6HPuAuCduh-a7O",
            },
        ],
    },
    TestCase {
        player: "5ec65609",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "4SvMpDQH-vBJCw" },
        ],
        sig: &[
            TestStep {
                input: "AAJAJfQdSswRQIhAMG5SN7-cAFChdrE7tLA6grH0rTMICA1mmDc0HoXgW3CAiAQQ4=CspfaF_vt82XH5yewvqcuEkvzeTsbRuHssRMyJQ=I",
                expected: "AJfQdSswRQIhAMG5SN7-cAFChdrE7tLA6grI0rTMICA1mmDc0HoXgW3CAiAQQ4HCspfaF_vt82XH5yewvqcuEkvzeTsbRuHssRMyJQ==",
            },
        ],
    },
    TestCase {
        player: "6742b2b9",
        variants: None,
        n: &[
            TestStep { input: "_HPB-7GFg1VTkn9u", expected: "qUAsPryAO_ByYg" },
            TestStep { input: "K1t_fcB6phzuq2SF", expected: "Y7PcOt3VE62mog" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "AJfQdSswRAIgMVVvrovTbw6UNh99kPa4D_XQjGT4qYu7S6SHM8EjoCACIEQnz-nKN5RgG6iUTnNJC58csYPSrnS_SzricuUMJZGM",
            },
        ],
    },
    TestCase {
        player: "23ccdd25",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "orSsTqUaUO-j" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "ZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hAU6wbTvorvVVMgIARwsSdQfJAN",
            },
        ],
    },
    TestCase {
        player: "3597727b",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "PRwo5dDfisg0ejA2" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "AAJfQdSswRAIgMVVvrovTbw6UNh99kPa4D_XQjGT4qYuMS6SHM8Ej7CACIEQnz-nKN5RgG6iUTnNJC58csYPSroS_SzricuUMJZG",
            },
        ],
    },
    TestCase {
        player: "3752a005",
        variants: Some(VARIANTS_NO_TCE),
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "j22ZtsqVsR0Dn" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "ZJM_ucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHG6S7uYq4TGjQXSD4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
            },
        ],
    },
    TestCase {
        player: "afc7785b",
        variants: Some(VARIANTS_NO_TCE),
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "j22ZtsqVsR0Dn" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "ZJM_ucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHG6S7uYq4TGjQXSD4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
            },
        ],
    },
    TestCase {
        player: "b9645327",
        variants: Some(VARIANTS_NO_TCE),
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "j22ZtsqVsR0Dn" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "ZJM_ucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHG6S7uYq4TGjQXSD4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
            },
        ],
    },
    TestCase {
        player: "035b9195",
        variants: Some(VARIANTS_NO_TCE),
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "j22ZtsqVsR0Dn" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "ZJM_ucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHG6S7uYq4TGjQXSD4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
            },
        ],
    },
    TestCase {
        player: "6740c111",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "AVsXYE0uE1k8e" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "JfQdSswRAIgMVVvrovTbw6UNh99kPa4D_XQjGT4qYu7S6SHM8EjoCACIEQnz-MKN5RgG6iUTnNJC58csYPSrnS_SzricuUMJZGn",
            },
        ],
    },
    TestCase {
        player: "f6a4f3bc",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "H1NKYFbhlqZ" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "JfQdSswRAIgMVVvrovTbw6UNh99kPa4D_XQjGT4qYM7S6SHM8EjoCACIEQnz-nKM5RgG6iUTnNJC58cNYPSrnS_SzricuUMJZGu",
            },
        ],
    },
    TestCase {
        player: "b66835e2",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "H1NKYFbhlqZ" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "JfQdSswRAIgMVVvrovTbw6UNh99kPa4D_XQjGT4qYM7S6SHM8EjoCACIEQnz-nKM5RgG6iUTnNJC58cNYPSrnS_SzricuUMJZGu",
            },
        ],
    },
    TestCase {
        player: "4f8fa943",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "JWWr7hDSRpMq5" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "AAJfQdSswRAIgMVVvrovTbw6UNh99kPa4D_XQjGT4qYu7S6SHr8EjoCACIEQnz-nKN5RgG6iUTnNZC58csYPSMnS_SzricuUM",
            },
        ],
    },
    TestCase {
        player: "0004de42",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "OPd7UEsCDmCw4qD0" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA",
                expected: "ZJMUucirzS_SnrSPYsc85MJNnTUi6GgR5NCn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQ",
            },
        ],
    },
    TestCase {
        player: "2b83d2e0",
        variants: None,
        n: &[
            TestStep { input: "0eRGgQWJGfT5rFHFj", expected: "euHbygrCMLksxd" },
        ],
        sig: &[
            TestStep {
                input: "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJA",
                expected: "-MGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKnMznQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJ",
            },
        ],
    },
    TestCase {
        player: "638ec5c6",
        variants: None,
        n: &[
            TestStep { input: "ZdZIqFPQK-Ty8wId", expected: "1qov8-KM-yH" },
        ],
        sig: &[
            TestStep {
                input: "gN7a-hudCuAuPH6fByOk1_GNXN0yNMHShjZXS2VOgsEItAJz0tipeavEOmNdYN-wUtcEqD3bCXjc0iyKfAyZxCBGgIARwsSdQfJ2CJtt",
                expected: "MhudCuAuP-6fByOk1_GNXN7gNHHShjyXS2VOgsEItAJz0tipeav0OmNdYN-wUtcEqD3bCXjc0iyKfAyZxCBGgIARwsSdQfJ2CJtt",
            },
        ],
    },
    TestCase {
        player: "87644c66",
        variants: None,
        n: &[
            TestStep { input: "ZdZIqFPQK-Ty8wId", expected: "iF5NxEm1BYk" },
        ],
        sig: &[
            TestStep {
                input: "gN7a-hudCuAuPH6fByOk1_GNXN0yNMHShjZXS2VOgsEItAJz0tipeavEOmNdYN-wUtcEqD3bCXjc0iyKfAyZxCBGgIARwsSdQfJ2CJtt",
                expected: "atJC2JfQdSswRAtgGBCxZyAfKyi0cjXCb3DqEctUw-NYdNmOEvIepit0zJAtIEsgOV2SXZjhSHMNy0NXNG_1kOyBf6HPuAuCduh-a7Ng",
            },
        ],
    },
];
