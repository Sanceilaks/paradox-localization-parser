use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct LocalizationUnit {
    pub key: String,
    pub version: Option<i32>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Localization {
    pub lang: String,
    pub units: Vec<LocalizationUnit>,
}

impl Localization {
    /// Parse a string containing Hearts of Iron IV localisation data into a vector of
    /// `Localization` objects.
    ///
    /// The string is expected to be a valid yml file as described by the Hearts of Iron IV
    /// game engine with the following structure:
    ///
    /// * Each line can be empty or start with a `#` to be considered a comment
    /// * Every other line is a language entry of the form `l_<language_code>`, for example
    ///   `l_english`
    /// * Every line after a language entry line is a unit entry of the form
    ///   `<localisation_key>:<version> "<localisation_value>"` until the next language entry
    ///   line is encountered. `<localisation_key>` can contain any characters, including
    ///   special characters like `.`. `<version>` is optional. The
    ///   `localisation_value` is enclosed in double quotes and can contain any characters
    ///   except for double quotes.
    ///
    /// The returned vector contains a `Localization` object for each language entry in the
    /// input string. The `units` field of each `Localization` object contains a `LocalizationUnit`
    /// object for each unit entry associated with the language entry.
    pub fn parse(content: impl AsRef<str>) -> Vec<Self> {
        let content = content.as_ref();
        let lines = content.lines().collect::<Vec<_>>();

        // every lang entry starts with l_
        let lang_entries: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                if line.starts_with("l_") {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        let mut units_for_entry: HashMap<&str, Vec<&str>> = HashMap::new();

        for (i, lang_entry_idx) in lang_entries.iter().enumerate() {
            let next_entry_idx = if i == lang_entries.len() - 1 {
                lines.len()
            } else {
                lang_entries[i + 1]
            };

            let associated_units = &lines[*lang_entry_idx + 1..next_entry_idx];

            units_for_entry.insert(&lines[*lang_entry_idx][2..], associated_units.to_vec());
        }

        let mut locals: Vec<Localization> = Vec::new();
        let localization_unit_regex =
            regex::Regex::new(r#"(?P<key>.*):(?P<version>\d+)?\s+\"(?P<content>.*)\"$"#).unwrap();
        for (lang, units) in units_for_entry {
            let header = lang.replace("l_", "");
            let mut local_units = Vec::new();

            for unit in units {
                // unit has the form 'localisation_key:0 "Localisation value"'
                // important: localisation_key can contain '.' or other special characters

                let unit = unit.trim();
                if unit.is_empty() || unit.starts_with("#") {
                    continue;
                }

                let caps = localization_unit_regex.captures(unit).unwrap();
                let key = caps.name("key").unwrap().as_str().to_string();
                let version = caps.name("version").map(|v| v.as_str().parse().unwrap());
                let value = caps.name("content").unwrap().as_str().to_string();
                local_units.push(LocalizationUnit {
                    key,
                    version,
                    value,
                });
            }

            locals.push(Localization {
                lang: header,
                units: local_units,
            });
        }

        locals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let content = r#"
l_russian:
 #News events
 wuw_GER_news.1.t: "Воссоединение Германии"
 wuw_GER_news.1.d: "Возрадуйся, немецкий народ! Долой старые, произвольно проведенные границы между регионами, служившие лишь одной цели — разлучить родных братьев! Наступает новая эпоха. Сегодня [FROM.GetLeader], достойный последователь великого Бисмарка, объявляет о воссоединении Германии и зарождении нового государства, которому суждено простоять тысячу лет!"
 wuw_GER_news.1.a: "Наконец-то Германия вновь едина"
 wuw_GER_news.1.b: "То ли еще будет"
 wuw_GER_news.2.t: "Восстание в Силезии"
 wuw_GER_news.2.desc: "Мятежные жители Немецкой Силезии в четвертый раз со дня подписания Версальского мирного договора взбунтовались против властей. [SIL.GetLeader] утверждает, что силезский народ взял в руки оружие, чтобы отстоять независимость своего государства, а заодно защититься от последствий гражданской войны в Германии. Это восстание разительно отличается от предыдущих, направленных на обретение автономии в составе польского государства. Время покажет, смогут ли повстанцы дать отпор немецкой армии."
 wuw_GER_news.2.a: "Любопытный поворот событий"
 wuw_GER_news.2.b: "Раньше у них ничего не получалось"
 wuw_GER_news.2.c: "Вот он, решающий миг!"
 wuw_GER_news.2.d: "Будем надеяться, что сейчас у них всё получится"
 #Austrian events
 AUS_political_events.1.t: "Судьба Creditanstalt-Bankverein"
 AUS_political_events.1.desc: "Нам стало известно, что произошедшее в 1934 году слияние банков, в результате которого прошедший повторную капитализацию Creditanstalt завладел всеми активами и обязательствами Wiener Bankverein, а также получил контроль над операциями Niederösterreichische Escompte-Gesellschaft, привело к негативным последствиям: экономическое развитие нашего государства замедлилось. Кроме того, мы в итоге передали весь финансовый сектор в руки монополиста. Поскольку регулирование этого сектора входит в наши обязанности, нам следует подумать, как действовать дальше."
 AUS_political_events.1.a: "Оставим все как есть: так будет надежнее всего."
 AUS_political_events.1.a_tt: "Наши промышленные концерны останутся без изменений. Creditanstalt-Bankverein останется доступен. Эффект:"
 AUS_political_events.1.b: "Österreichische Creditanstalt — вот лучший выбор."
 AUS_political_events.1.c: "Wiener Bankverein нас не подведет."
 AUS_reestablish_creditanstalt: "Creditanstalt-Bankverein прекратит деятельность, а новообразованный Österreichische Creditanstalt получит полную поддержку."
 AUS_reestablish_benkverein: "Creditanstalt-Bankverein прекратит деятельность, а новообразованный Wiener Bankverein получит полную поддержку."
 AUS_political_events.2.t: "Скандал со страховой компанией Phönix"
 AUS_political_events.2.desc: "Вскоре после смерти Вильгельма Берлинера — пользовавшегося любовью всех сотрудников руководителя страховой компании Phönix, его детище столкнулось с новой бедой. Новый руководитель Эберхард Райнингхаус представил правительству доказательства подделки финансовых отчетов. Оказалось, что еще с 1929 года за компанией тянулся дефицит, достигший уже 250 миллионов шиллингов.\n\nКомпания объявила о банкротстве и тут же прекратила свою деятельность. При этом была закрыта широкая программа страхования участников боевых действий, что вызвало массовое недовольство иждивенцев, которые лишились средств к существованию."
 AUS_political_events.2.a: "Какой кошмар!"
"#;
        let localization = Localization::parse(content);
        println!("{:#?}", localization);

        assert_eq!(localization.len(), 1);

        let first = localization.first().unwrap();
        assert_eq!(first.units.len(), 21);
        assert_eq!(first.lang, "russian");
    }
}
