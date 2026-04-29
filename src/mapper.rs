#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyStroke {
    pub code: u16,
    pub shift: bool,
}

impl KeyStroke {
    #[inline]
    pub const fn new(code: u16, shift: bool) -> Self {
        Self { code, shift }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StrokeSeq {
    pub strokes: [KeyStroke; 2],
    pub len: u8,
}

impl StrokeSeq {
    #[inline]
    pub fn single(stroke: KeyStroke) -> Self {
        Self {
            strokes: [stroke, KeyStroke::new(0, false)],
            len: 1,
        }
    }

    #[inline]
    pub fn double(first: KeyStroke, second: KeyStroke) -> Self {
        Self {
            strokes: [first, second],
            len: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    FaroeseToUkrainian,
    UkrainianToFaroese,
}

pub struct LayoutMapper;

impl LayoutMapper {
    #[inline]
    pub fn detect_direction(text: &str) -> Direction {
        let mut ukr_count = 0u32;
        let mut fo_count = 0u32;

        for char_item in text.chars() {
            if char_to_ua_strokes(char_item).is_some() {
                ukr_count += 1;
            } else if char_to_fo_strokes(char_item).is_some() {
                fo_count += 1;
            }
        }

        if ukr_count > fo_count {
            Direction::UkrainianToFaroese
        } else {
            Direction::FaroeseToUkrainian
        }
    }

    #[inline]
    pub fn decompose(char_item: char, dir: Direction) -> Option<StrokeSeq> {
        match dir {
            Direction::UkrainianToFaroese => char_to_ua_strokes(char_item),
            Direction::FaroeseToUkrainian => char_to_fo_strokes(char_item),
        }
    }

    #[inline]
    pub fn synthesize(stroke: KeyStroke, dir: Direction) -> Option<char> {
        match dir {
            Direction::UkrainianToFaroese => stroke_to_fo(stroke),
            Direction::FaroeseToUkrainian => stroke_to_ua(stroke),
        }
    }

    pub fn translate(text: &str, dir: Direction) -> String {
        let mut result = String::with_capacity(text.len());

        for char_item in text.chars() {
            if let Some(seq) = Self::decompose(char_item, dir) {
                match dir {
                    Direction::FaroeseToUkrainian => {
                        for idx in 0..seq.len as usize {
                            if let Some(mapped) = Self::synthesize(seq.strokes[idx], dir) {
                                result.push(mapped);
                            }
                        }
                    }
                    Direction::UkrainianToFaroese => {
                        let mut pending_dead: Option<KeyStroke> = None;
                        for idx in 0..seq.len as usize {
                            let stroke = seq.strokes[idx];

                            if let Some(dead) = pending_dead.take() {
                                if let Some(composed) = compose_fo_accent(dead, stroke) {
                                    result.push(composed);
                                    continue;
                                }
                                if let Some(mapped) = Self::synthesize(dead, dir) {
                                    result.push(mapped);
                                }
                            }

                            if is_fo_accent_key(stroke) {
                                pending_dead = Some(stroke);
                            } else if let Some(mapped) = Self::synthesize(stroke, dir) {
                                result.push(mapped);
                            }
                        }
                        if let Some(dead) = pending_dead {
                            if let Some(mapped) = Self::synthesize(dead, dir) {
                                result.push(mapped);
                            }
                        }
                    }
                }
            } else {
                result.push(char_item);
            }
        }

        result
    }
}

#[inline]
fn is_fo_accent_key(stroke: KeyStroke) -> bool {
    !stroke.shift && (stroke.code == 13 || stroke.code == 27)
}

fn compose_fo_accent(dead: KeyStroke, base: KeyStroke) -> Option<char> {
    if dead.shift {
        return None;
    }

    match dead.code {
        13 => compose_acute(base),
        27 => compose_diaeresis(base),
        _ => None,
    }
}

fn compose_acute(base: KeyStroke) -> Option<char> {
    if !base.shift {
        match base.code {
            30 => Some('á'), 18 => Some('é'), 23 => Some('í'),
            24 => Some('ó'), 22 => Some('ú'), 21 => Some('ý'),
            _ => None,
        }
    } else {
        match base.code {
            30 => Some('Á'), 18 => Some('É'), 23 => Some('Í'),
            24 => Some('Ó'), 22 => Some('Ú'), 21 => Some('Ý'),
            _ => None,
        }
    }
}

fn compose_diaeresis(base: KeyStroke) -> Option<char> {
    if !base.shift {
        match base.code {
            30 => Some('ä'), 18 => Some('ë'), 23 => Some('ï'),
            24 => Some('ö'), 22 => Some('ü'), 21 => Some('ÿ'),
            _ => None,
        }
    } else {
        match base.code {
            30 => Some('Ä'), 18 => Some('Ë'), 23 => Some('Ï'),
            24 => Some('Ö'), 22 => Some('Ü'), 21 => Some('Ÿ'),
            _ => None,
        }
    }
}

fn char_to_ua_strokes(char_item: char) -> Option<StrokeSeq> {
    match char_item {
        'й' => Some(StrokeSeq::single(KeyStroke::new(16, false))),
        'ц' => Some(StrokeSeq::single(KeyStroke::new(17, false))),
        'у' => Some(StrokeSeq::single(KeyStroke::new(18, false))),
        'к' => Some(StrokeSeq::single(KeyStroke::new(19, false))),
        'е' => Some(StrokeSeq::single(KeyStroke::new(20, false))),
        'н' => Some(StrokeSeq::single(KeyStroke::new(21, false))),
        'г' => Some(StrokeSeq::single(KeyStroke::new(22, false))),
        'ш' => Some(StrokeSeq::single(KeyStroke::new(23, false))),
        'щ' => Some(StrokeSeq::single(KeyStroke::new(24, false))),
        'з' => Some(StrokeSeq::single(KeyStroke::new(25, false))),
        'х' => Some(StrokeSeq::single(KeyStroke::new(26, false))),
        'ї' => Some(StrokeSeq::single(KeyStroke::new(27, false))),
        'ф' => Some(StrokeSeq::single(KeyStroke::new(30, false))),
        'і' => Some(StrokeSeq::single(KeyStroke::new(31, false))),
        'в' => Some(StrokeSeq::single(KeyStroke::new(32, false))),
        'а' => Some(StrokeSeq::single(KeyStroke::new(33, false))),
        'п' => Some(StrokeSeq::single(KeyStroke::new(34, false))),
        'р' => Some(StrokeSeq::single(KeyStroke::new(35, false))),
        'о' => Some(StrokeSeq::single(KeyStroke::new(36, false))),
        'л' => Some(StrokeSeq::single(KeyStroke::new(37, false))),
        'д' => Some(StrokeSeq::single(KeyStroke::new(38, false))),
        'ж' => Some(StrokeSeq::single(KeyStroke::new(39, false))),
        'є' => Some(StrokeSeq::single(KeyStroke::new(40, false))),
        'ґ' => Some(StrokeSeq::single(KeyStroke::new(43, false))),
        'я' => Some(StrokeSeq::single(KeyStroke::new(44, false))),
        'ч' => Some(StrokeSeq::single(KeyStroke::new(45, false))),
        'с' => Some(StrokeSeq::single(KeyStroke::new(46, false))),
        'м' => Some(StrokeSeq::single(KeyStroke::new(47, false))),
        'и' => Some(StrokeSeq::single(KeyStroke::new(48, false))),
        'т' => Some(StrokeSeq::single(KeyStroke::new(49, false))),
        'ь' => Some(StrokeSeq::single(KeyStroke::new(50, false))),
        'б' => Some(StrokeSeq::single(KeyStroke::new(51, false))),
        'ю' => Some(StrokeSeq::single(KeyStroke::new(52, false))),

        'Й' => Some(StrokeSeq::single(KeyStroke::new(16, true))),
        'Ц' => Some(StrokeSeq::single(KeyStroke::new(17, true))),
        'У' => Some(StrokeSeq::single(KeyStroke::new(18, true))),
        'К' => Some(StrokeSeq::single(KeyStroke::new(19, true))),
        'Е' => Some(StrokeSeq::single(KeyStroke::new(20, true))),
        'Н' => Some(StrokeSeq::single(KeyStroke::new(21, true))),
        'Г' => Some(StrokeSeq::single(KeyStroke::new(22, true))),
        'Ш' => Some(StrokeSeq::single(KeyStroke::new(23, true))),
        'Щ' => Some(StrokeSeq::single(KeyStroke::new(24, true))),
        'З' => Some(StrokeSeq::single(KeyStroke::new(25, true))),
        'Х' => Some(StrokeSeq::single(KeyStroke::new(26, true))),
        'Ї' => Some(StrokeSeq::single(KeyStroke::new(27, true))),
        'Ф' => Some(StrokeSeq::single(KeyStroke::new(30, true))),
        'І' => Some(StrokeSeq::single(KeyStroke::new(31, true))),
        'В' => Some(StrokeSeq::single(KeyStroke::new(32, true))),
        'А' => Some(StrokeSeq::single(KeyStroke::new(33, true))),
        'П' => Some(StrokeSeq::single(KeyStroke::new(34, true))),
        'Р' => Some(StrokeSeq::single(KeyStroke::new(35, true))),
        'О' => Some(StrokeSeq::single(KeyStroke::new(36, true))),
        'Л' => Some(StrokeSeq::single(KeyStroke::new(37, true))),
        'Д' => Some(StrokeSeq::single(KeyStroke::new(38, true))),
        'Ж' => Some(StrokeSeq::single(KeyStroke::new(39, true))),
        'Є' => Some(StrokeSeq::single(KeyStroke::new(40, true))),
        'Ґ' => Some(StrokeSeq::single(KeyStroke::new(43, true))),
        'Я' => Some(StrokeSeq::single(KeyStroke::new(44, true))),
        'Ч' => Some(StrokeSeq::single(KeyStroke::new(45, true))),
        'С' => Some(StrokeSeq::single(KeyStroke::new(46, true))),
        'М' => Some(StrokeSeq::single(KeyStroke::new(47, true))),
        'И' => Some(StrokeSeq::single(KeyStroke::new(48, true))),
        'Т' => Some(StrokeSeq::single(KeyStroke::new(49, true))),
        'Ь' => Some(StrokeSeq::single(KeyStroke::new(50, true))),
        'Б' => Some(StrokeSeq::single(KeyStroke::new(51, true))),
        'Ю' => Some(StrokeSeq::single(KeyStroke::new(52, true))),
        _ => None,
    }
}

fn char_to_fo_strokes(char_item: char) -> Option<StrokeSeq> {
    match char_item {
        'q' => Some(StrokeSeq::single(KeyStroke::new(16, false))),
        'w' => Some(StrokeSeq::single(KeyStroke::new(17, false))),
        'e' => Some(StrokeSeq::single(KeyStroke::new(18, false))),
        'r' => Some(StrokeSeq::single(KeyStroke::new(19, false))),
        't' => Some(StrokeSeq::single(KeyStroke::new(20, false))),
        'y' => Some(StrokeSeq::single(KeyStroke::new(21, false))),
        'u' => Some(StrokeSeq::single(KeyStroke::new(22, false))),
        'i' => Some(StrokeSeq::single(KeyStroke::new(23, false))),
        'o' => Some(StrokeSeq::single(KeyStroke::new(24, false))),
        'p' => Some(StrokeSeq::single(KeyStroke::new(25, false))),
        'å' => Some(StrokeSeq::single(KeyStroke::new(26, false))),
        '¨' => Some(StrokeSeq::single(KeyStroke::new(27, false))),
        'a' => Some(StrokeSeq::single(KeyStroke::new(30, false))),
        's' => Some(StrokeSeq::single(KeyStroke::new(31, false))),
        'd' => Some(StrokeSeq::single(KeyStroke::new(32, false))),
        'f' => Some(StrokeSeq::single(KeyStroke::new(33, false))),
        'g' => Some(StrokeSeq::single(KeyStroke::new(34, false))),
        'h' => Some(StrokeSeq::single(KeyStroke::new(35, false))),
        'j' => Some(StrokeSeq::single(KeyStroke::new(36, false))),
        'k' => Some(StrokeSeq::single(KeyStroke::new(37, false))),
        'l' => Some(StrokeSeq::single(KeyStroke::new(38, false))),
        'æ' => Some(StrokeSeq::single(KeyStroke::new(39, false))),
        'ø' => Some(StrokeSeq::single(KeyStroke::new(40, false))),
        '\'' => Some(StrokeSeq::single(KeyStroke::new(43, false))),
        'z' => Some(StrokeSeq::single(KeyStroke::new(44, false))),
        'x' => Some(StrokeSeq::single(KeyStroke::new(45, false))),
        'c' => Some(StrokeSeq::single(KeyStroke::new(46, false))),
        'v' => Some(StrokeSeq::single(KeyStroke::new(47, false))),
        'b' => Some(StrokeSeq::single(KeyStroke::new(48, false))),
        'n' => Some(StrokeSeq::single(KeyStroke::new(49, false))),
        'm' => Some(StrokeSeq::single(KeyStroke::new(50, false))),

        'Q' => Some(StrokeSeq::single(KeyStroke::new(16, true))),
        'W' => Some(StrokeSeq::single(KeyStroke::new(17, true))),
        'E' => Some(StrokeSeq::single(KeyStroke::new(18, true))),
        'R' => Some(StrokeSeq::single(KeyStroke::new(19, true))),
        'T' => Some(StrokeSeq::single(KeyStroke::new(20, true))),
        'Y' => Some(StrokeSeq::single(KeyStroke::new(21, true))),
        'U' => Some(StrokeSeq::single(KeyStroke::new(22, true))),
        'I' => Some(StrokeSeq::single(KeyStroke::new(23, true))),
        'O' => Some(StrokeSeq::single(KeyStroke::new(24, true))),
        'P' => Some(StrokeSeq::single(KeyStroke::new(25, true))),
        'Å' => Some(StrokeSeq::single(KeyStroke::new(26, true))),
        '^' => Some(StrokeSeq::single(KeyStroke::new(27, true))),
        'A' => Some(StrokeSeq::single(KeyStroke::new(30, true))),
        'S' => Some(StrokeSeq::single(KeyStroke::new(31, true))),
        'D' => Some(StrokeSeq::single(KeyStroke::new(32, true))),
        'F' => Some(StrokeSeq::single(KeyStroke::new(33, true))),
        'G' => Some(StrokeSeq::single(KeyStroke::new(34, true))),
        'H' => Some(StrokeSeq::single(KeyStroke::new(35, true))),
        'J' => Some(StrokeSeq::single(KeyStroke::new(36, true))),
        'K' => Some(StrokeSeq::single(KeyStroke::new(37, true))),
        'L' => Some(StrokeSeq::single(KeyStroke::new(38, true))),
        'Æ' => Some(StrokeSeq::single(KeyStroke::new(39, true))),
        'Ø' => Some(StrokeSeq::single(KeyStroke::new(40, true))),
        '*' => Some(StrokeSeq::single(KeyStroke::new(43, true))),
        'Z' => Some(StrokeSeq::single(KeyStroke::new(44, true))),
        'X' => Some(StrokeSeq::single(KeyStroke::new(45, true))),
        'C' => Some(StrokeSeq::single(KeyStroke::new(46, true))),
        'V' => Some(StrokeSeq::single(KeyStroke::new(47, true))),
        'B' => Some(StrokeSeq::single(KeyStroke::new(48, true))),
        'N' => Some(StrokeSeq::single(KeyStroke::new(49, true))),
        'M' => Some(StrokeSeq::single(KeyStroke::new(50, true))),

        'á' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(30, false))),
        'é' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(18, false))),
        'í' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(23, false))),
        'ó' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(24, false))),
        'ú' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(22, false))),
        'ý' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(21, false))),
        'Á' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(30, true))),
        'É' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(18, true))),
        'Í' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(23, true))),
        'Ó' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(24, true))),
        'Ú' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(22, true))),
        'Ý' => Some(StrokeSeq::double(KeyStroke::new(13, false), KeyStroke::new(21, true))),

        'ä' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(30, false))),
        'ë' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(18, false))),
        'ï' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(23, false))),
        'ö' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(24, false))),
        'ü' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(22, false))),
        'ÿ' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(21, false))),
        'Ä' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(30, true))),
        'Ë' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(18, true))),
        'Ï' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(23, true))),
        'Ö' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(24, true))),
        'Ü' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(22, true))),
        'Ÿ' => Some(StrokeSeq::double(KeyStroke::new(27, false), KeyStroke::new(21, true))),
        _ => None,
    }
}

#[inline]
fn stroke_to_ua(stroke: KeyStroke) -> Option<char> {
    if stroke.shift {
        match stroke.code {
            16 => Some('Й'), 17 => Some('Ц'), 18 => Some('У'), 19 => Some('К'),
            20 => Some('Е'), 21 => Some('Н'), 22 => Some('Г'), 23 => Some('Ш'),
            24 => Some('Щ'), 25 => Some('З'), 26 => Some('Х'), 27 => Some('Ї'),
            30 => Some('Ф'), 31 => Some('І'), 32 => Some('В'), 33 => Some('А'),
            34 => Some('П'), 35 => Some('Р'), 36 => Some('О'), 37 => Some('Л'),
            38 => Some('Д'), 39 => Some('Ж'), 40 => Some('Є'),
            43 => Some('Ґ'),
            44 => Some('Я'), 45 => Some('Ч'), 46 => Some('С'), 47 => Some('М'),
            48 => Some('И'), 49 => Some('Т'), 50 => Some('Ь'), 51 => Some('Б'),
            52 => Some('Ю'),
            _ => None,
        }
    } else {
        match stroke.code {
            16 => Some('й'), 17 => Some('ц'), 18 => Some('у'), 19 => Some('к'),
            20 => Some('е'), 21 => Some('н'), 22 => Some('г'), 23 => Some('ш'),
            24 => Some('щ'), 25 => Some('з'), 26 => Some('х'), 27 => Some('ї'),
            30 => Some('ф'), 31 => Some('і'), 32 => Some('в'), 33 => Some('а'),
            34 => Some('п'), 35 => Some('р'), 36 => Some('о'), 37 => Some('л'),
            38 => Some('д'), 39 => Some('ж'), 40 => Some('є'),
            43 => Some('ґ'),
            44 => Some('я'), 45 => Some('ч'), 46 => Some('с'), 47 => Some('м'),
            48 => Some('и'), 49 => Some('т'), 50 => Some('ь'), 51 => Some('б'),
            52 => Some('ю'),
            _ => None,
        }
    }
}

#[inline]
fn stroke_to_fo(stroke: KeyStroke) -> Option<char> {
    if stroke.shift {
        match stroke.code {
            16 => Some('Q'), 17 => Some('W'), 18 => Some('E'), 19 => Some('R'),
            20 => Some('T'), 21 => Some('Y'), 22 => Some('U'), 23 => Some('I'),
            24 => Some('O'), 25 => Some('P'), 26 => Some('Å'), 27 => Some('^'),
            30 => Some('A'), 31 => Some('S'), 32 => Some('D'), 33 => Some('F'),
            34 => Some('G'), 35 => Some('H'), 36 => Some('J'), 37 => Some('K'),
            38 => Some('L'), 39 => Some('Æ'), 40 => Some('Ø'),
            43 => Some('*'),
            44 => Some('Z'), 45 => Some('X'), 46 => Some('C'), 47 => Some('V'),
            48 => Some('B'), 49 => Some('N'), 50 => Some('M'),
            _ => None,
        }
    } else {
        match stroke.code {
            13 => Some('´'),
            16 => Some('q'), 17 => Some('w'), 18 => Some('e'), 19 => Some('r'),
            20 => Some('t'), 21 => Some('y'), 22 => Some('u'), 23 => Some('i'),
            24 => Some('o'), 25 => Some('p'), 26 => Some('å'), 27 => Some('¨'),
            30 => Some('a'), 31 => Some('s'), 32 => Some('d'), 33 => Some('f'),
            34 => Some('g'), 35 => Some('h'), 36 => Some('j'), 37 => Some('k'),
            38 => Some('l'), 39 => Some('æ'), 40 => Some('ø'),
            43 => Some('\''),
            44 => Some('z'), 45 => Some('x'), 46 => Some('c'), 47 => Some('v'),
            48 => Some('b'), 49 => Some('n'), 50 => Some('m'),
            _ => None,
        }
    }
}
