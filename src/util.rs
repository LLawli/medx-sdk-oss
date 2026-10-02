//! Utilitários compartilhados entre os módulos do SDK.

use chrono::Local;

/// Retorna a data/hora **local** atual formatada como `"YYYY-MM-DDTHH:MM:SS"`.
///
/// A plataforma MedX trabalha com horário local naive (o frontend envia
/// `kendo.toString(picker, 'yyyy-MM-ddTHH:mm:ss')` e `moment().format()` com o
/// offset removido). Usar UTC aqui deslocaria notas, logs e mensagens em até
/// algumas horas para clínicas fora de GMT. A precisão é de segundos, sem
/// timezone no texto.
pub fn current_datetime_str() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Codifica um valor de texto para a query string (codificação de
/// formulário, como o webapp e o ASP.NET da MedX esperam): `A-Z a-z 0-9 - _
/// . ~` ficam como estão, espaço vira `+`, e todo o resto vira `%XX` dos
/// bytes UTF-8. Use em todo valor de texto que entra numa query.
pub fn encode_query_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_datetime_str_formato_correto() {
        let dt = current_datetime_str();
        assert_eq!(dt.len(), 19, "data/hora deve ter 19 chars: '{dt}'");
        assert_eq!(&dt[10..11], "T", "posição 10 deve ser 'T'");
        assert_eq!(&dt[4..5], "-");
        assert_eq!(&dt[7..8], "-");
        assert_eq!(&dt[13..14], ":");
        assert_eq!(&dt[16..17], ":");
    }
}
