use super::*;
// use ratatui::style::Style;

// #[test]
// fn render() {
//     let app = App::default();
//     let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

//     app.render(buf.area, &mut buf);

//     let mut expected = Buffer::with_lines(vec![
//         "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
//         "┃                    Value: 0                    ┃",
//         "┃                                                ┃",
//         "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
//     ]);
//     let title_style = Style::new().bold();
//     let counter_style = Style::new().yellow();
//     let key_style = Style::new().blue().bold();
//     expected.set_style(Rect::new(14, 0, 22, 1), title_style);
//     expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
//     expected.set_style(Rect::new(13, 3, 6, 1), key_style);
//     expected.set_style(Rect::new(30, 3, 7, 1), key_style);
//     expected.set_style(Rect::new(43, 3, 4, 1), key_style);

//     assert_eq!(buf, expected);
// }

#[test]
fn location_compares() -> io::Result<()> {
    assert!(Location::new(2, 2) < Location::new(3, 3));
    assert!(
        Location::new(2, 2)
            .partial_cmp(&Location::new(3, 3))
            .is_some_and(|x| x.is_lt())
    );
    assert!(Location::new(2, 2) < Location::new(2, 3));
    assert!(Location::new(2, 2) < Location::new(3, 2));
    assert!(Location::new(2, 2) == Location::new(2, 2));
    assert!(Location::new(2, 2) > Location::new(2, 1));
    assert!(Location::new(2, 2) > Location::new(1, 2));
    assert!(
        Location::new(0, 3)
            .partial_cmp(&Location::new(1, 2))
            .is_none()
    );
    assert!(
        Location::new(1, 2)
            .partial_cmp(&Location::new(0, 3))
            .is_none()
    );

    Ok(())
}

#[test]
fn handle_key_event() -> io::Result<()> {
    let mut app = App::default();
    app.handle_key_event(KeyCode::Char('q').into());
    assert!(app.exit);

    Ok(())
}
