use larp_script::interpreter::{Interpreter, LarpError};

fn run(source: &str) -> Result<Vec<String>, LarpError> {
    let mut interpreter = Interpreter::new();
    interpreter.run_source(source)?;
    Ok(interpreter.take_output())
}

#[test]
fn grind_over_a_number_counts_from_zero() {
    assert_eq!(run("grind i in 3 { yap(i); }").unwrap(), ["0", "1", "2"]);
}

#[test]
fn grind_over_zero_runs_no_iterations() {
    assert!(run("grind i in 0 { yap(i); }").unwrap().is_empty());
}

#[test]
fn grind_over_a_string_yields_each_character() {
    assert_eq!(run(r#"grind c in "rizz" { yap(c); }"#).unwrap(), ["r", "i", "z", "z"]);
}

#[test]
fn bet_and_clown_work_inside_grind() {
    let source = r#"
        grind i in 10 {
            if i == 1 { clown; }
            if i == 4 { bet; }
            yap(i);
        }
    "#;
    assert_eq!(run(source).unwrap(), ["0", "2", "3"]);
}

#[test]
fn mog_returns_from_inside_grind() {
    let source = r#"
        fun first_over(limit) {
            grind i in 100 {
                if i > limit { mog i; }
            }
            mog nah;
        }
        yap(first_over(41));
    "#;
    assert_eq!(run(source).unwrap(), ["42"]);
}

#[test]
fn loop_variable_does_not_leak_out_of_grind() {
    assert!(matches!(run("grind i in 2 { } yap(i);"), Err(LarpError::Runtime(_))));
}

#[test]
fn grind_rejects_negative_and_fractional_numbers() {
    assert!(matches!(run("grind i in -1 { }"), Err(LarpError::Runtime(_))));
    assert!(matches!(run("grind i in 2.5 { }"), Err(LarpError::Runtime(_))));
}
