//! Texto do modelo para o HTML que o prontuário da MedX guarda.

use medx_mcp::tools::text_to_html;

#[test]
fn uma_linha_vira_um_paragrafo() {
    assert_eq!(
        text_to_html("Paciente estável."),
        "<p>Paciente estável.</p>"
    );
}

#[test]
fn linha_em_branco_separa_paragrafos_e_quebra_simples_vira_br() {
    assert_eq!(text_to_html("a\nb\n\nc"), "<p>a<br>b</p><p>c</p>");
    assert_eq!(text_to_html("a\r\nb\r\n\r\nc"), "<p>a<br>b</p><p>c</p>");
    assert_eq!(text_to_html("a\n\n\n   \n\nb"), "<p>a</p><p>b</p>");
}

#[test]
fn espacos_nas_pontas_somem() {
    assert_eq!(text_to_html("  \n a \n "), "<p>a</p>");
    assert_eq!(text_to_html("   "), "");
}

#[test]
fn caracteres_de_html_sao_escapados() {
    assert_eq!(
        text_to_html("<script>x</script> & 1 < 2 \"ok\""),
        "<p>&lt;script&gt;x&lt;/script&gt; &amp; 1 &lt; 2 &quot;ok&quot;</p>"
    );
}
