use std::borrow::Cow;

use lazy_regex::regex_replace_all;

/// Map fullwidth Unicode variants to their ASCII equivalents so rules
/// written for ASCII match Japanese release names (`第０１話（BD）`):
/// digits ０-９, brackets （）［］, space U+3000 and hyphen －.
/// Returns a borrowed slice when nothing changed.
pub(crate) fn normalize_fullwidth(name: &str) -> Cow<'_, str> {
    if !name.chars().any(fullwidth_char) {
        return Cow::Borrowed(name);
    }
    Cow::Owned(name.chars().map(ascii_of_fullwidth).collect())
}

fn fullwidth_char(c: char) -> bool {
    matches!(
        c,
        '０'..='９' | '（' | '）' | '［' | '］' | '\u{3000}' | '－'
    )
}

fn ascii_of_fullwidth(c: char) -> char {
    match c {
        // Fullwidth digits U+FF10..U+FF19 map directly onto ASCII digits.
        '０'..='９' => char::from_digit(c as u32 - '０' as u32, 10).unwrap_or(c),
        '（' => '(',
        '）' => ')',
        '［' => '[',
        '］' => ']',
        '\u{3000}' => ' ',
        '－' => '-',
        _ => c,
    }
}

/// Strip release-group / technical tags from a series directory name,
/// leaving the bare title. Removes bracketed tags (`[1080p]`, `[HEVC]`,
/// `[VCB-Studio]`, ...), year markers, and trailing extensions, then
/// collapses separators into single ASCII spaces.
#[must_use]
pub fn series_title_from_name(name: &str) -> String {
    regex_replace_all!(
        r"\(\d{4}-\d{2}-\d{2}\)|\[\d{1,2}-\d{2}\]|\[\d{4}-\d{4}\]|\[(Part|PART) (-?\d)*\]|[\[\(]\d{4}[\]\)]|[\[\(]([\+_ -\.]?(x264|x264_10bit|1920(x|X)1080|Ma10p_1080p|(480|720|1080|2160)(p|P)|(2|4|8)(k|K)|JPSC|CR|OVA|ova|TV|tv|GB|gb|BD|bd|dvdrip|DVDRip|DVDrip|DVDRIP|BDRIP|BDrip|BDRip|P10|WebRip|webrip|AAC|ASSx2|HEVC|hevc|10(bit|BIT)|opus|OPUS|OPUSx2|AAC|ALAC|FLAC|AC3|MKV|mkv|MP4|mp4|ALL|all|Multi-Subs|Multiple Subtitle))*[\]\)]|\[\d{1,2}-\d{2}(TV|tv)?(全集)?(\+(合集版|SP|特典映像|剧场版|OAD|OVA))*\]|\[(flac|FLAC)\]|\[(flac|FLAC)(x|X)2\]|\[(HEVC|hevc)((_(x|X)265)|-10(bit|Bit|BIT))\]|\[(简繁外挂|日英双语|简体内嵌|简繁日双语外挂|简繁内封)\]|\[带配音音轨\]|\[HKG\]|\[DHR(&LKSUB)?\]|\[Comicat\]|\[CheeseAni\]|\[VCB-Studio\]|\[Snow-Raws\]|\[Nekomoe kissaten(&LoliHouse)?\]|\[hyakuhuyu(&LoliHouse)?\]|\[SAIO-Raws\]|\[Erai-raws\]|\[DBD-Raws\]|\[Fyy Raws\]|\[(DBD|dbd)制作组\]|\[路基艾尔\]|\[Sakurato&7³ACG\]|\[(Rev|rev)\]|\[(GB|gb)&?(BIG5|big5)\]|\[343-Labs\]|\.(mkv|MKV|mp4|MP4)|(TV|tv)|\.(OVA|ova)|1920(x|X)1080",
        name,
        ""
    )
    .replace('_', " ")
    .replace(['[', ']'], "")
    .trim()
    .to_string()
}
