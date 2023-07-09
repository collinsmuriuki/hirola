use hirola_core::TemplateResult;
use hirola_macros::html;

// fn entry(entry: u8) -> String {
//     html_to_string! {
//         <li>{entry}</li>
//     }
// }

#[test]
fn test() {
    let world = "planet";

    assert_eq!(
        html! {
            <p>{world}</p>
        },
        TemplateResult::empty()
    );
}
