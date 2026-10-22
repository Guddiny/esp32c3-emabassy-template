# Example of project for ESP32C3 + Rust + embassy 

Work still in the progress....

<img src="assets/All.png" alt="Web View example">

Web UI can be built directly from Rust code:
```rust
fn build(b: &mut Builder, settings: &mut Settings) {
    b.tab("Main", false, |b| {
        b.group("", false, Alignment::Vertical, |b| {
            b.group("wifi", false, Alignment::Vertical, |b| {
                b.input(
                    "SSID",
                    "SSID",
                    InputType::Text,
                    &mut String::try_from("K92_new").unwrap(),
                );
                b.input(
                    "Password",
                    "Password",
                    InputType::Password,
                    &mut String::try_from("123").unwrap(),
                );

                b.button("Connect", "Connect Btn");
            });

            b.group("User settings", false, Alignment::Vertical, |b| {
                b.toggle("Echo Mode", &mut true).notify_on_change();
                b.button("Connect", "Connect Btn");

                b.group("Email", false, Alignment::Horizontal, |b| {
                    b.input(
                        "Email",
                        "Email",
                        InputType::Email,
                        &mut String::try_from("my@email.com").unwrap(),
                    );
                    b.button("Add", "Add Email");
                });
            });
        });
    })
}
```
