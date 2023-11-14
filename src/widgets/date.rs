use gtk::prelude::*;

pub fn date() -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.style_context().add_class("date");

    let label = gtk::Label::new(None);
    container.add(&label);

    glib::MainContext::default().spawn_local(async move {
        loop {
            let time = chrono::Local::now();
            // Precise timing. Not sure how to do this in glib async yet
            // let next_time = time.duration_trunc(chrono::Duration::seconds(1)).unwrap() + Duration::from_secs(1);
            // eprintln!("Now:  {time:?}\nNext: {next_time:?}");
            label.set_label(&time.format("%H:%M:%S %Y/%m/%d").to_string());
            glib::timeout_future_seconds(1).await;
        }
    });

    return container;
}
