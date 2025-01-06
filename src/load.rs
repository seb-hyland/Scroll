use crate::{
    prelude::*,
    tools::scroll_processor,
};
use homedir::my_home;



#[component]
pub fn Loader() -> Element {
    let resource = use_resource(|| async {
        spawn(async {
            preload().await;
        });
    });

    match resource.read_unchecked().clone() {
        None => {
            rsx! { "Loading..." }
        }
        Some(_) => {
            rsx! { "Successfully loaded!" }
        }
    }
}


async fn preload() {
    let result: Result<(), String> = (|| { 
        compute_DOC_DIR()?;
        compute_DATABASE_HOLD()?;
        Ok(())
    })();
    match result {
        Ok(_) => {
            let nav = navigator();
            nav.push(Route::Viewer {});
        },
        Err(e) => {
            let nav = navigator();
            nav.push(Route::Purgatory { error: e.clone() });
        }
    }
}


fn compute_DOC_DIR() -> Result<(), String> {
    let err_msg = "DOC_DIR could not be initialized. Error: ";
    let home_dir: PathBuf = my_home()
        .map_err(|e| format!("{err_msg}{}", e.to_string()))?
        .ok_or(format!("{err_msg}"))?;
    let doc_dir = home_dir.join("Documents/iGEM-2025");

    let static_binding = LazyLock::force(&DOC_DIR);
    {
        let mut guard = static_binding
            .write()
            .map_err(|e| format!("{err_msg}{}", e.to_string()))?;
        *guard = doc_dir;
    }
    Ok(())
}


fn compute_DATABASE_HOLD() -> Result<(), String> {
    let err_msg = "DOC_DIR could not be initialized. Error: ";
    let database = scroll_processor::parse_all_databases()
        .map_err(|e| format!("{err_msg}{}", e.to_string()))?;

    let static_binding = LazyLock::force(&DATABASE_HOLD);
    {
        let mut guard = static_binding
            .write()
            .map_err(|e| format!("{err_msg}{}", e.to_string()))?;
        *guard = database;
    }

    Ok(())
}
