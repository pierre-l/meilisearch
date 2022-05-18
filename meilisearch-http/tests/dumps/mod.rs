mod data;

use crate::common::{default_settings, Server};
use meilisearch_http::Opt;
use serde_json::json;

#[actix_rt::test]
async fn get_unexisting_dump_status() {
    let server = Server::new().await;

    let (response, code) = server.get_dump_status("foobar").await;
    assert_eq!(code, 404);

    let expected_response = json!({
    "message": "Dump `foobar` not found.",
    "code": "dump_not_found",
    "type": "invalid_request",
    "link": "https://docs.meilisearch.com/errors#dump_not_found"
    });

    assert_eq!(response, expected_response);
}

// all the following test are ignored on windows. See #2364
#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v1() {
    let temp = tempfile::tempdir().unwrap();

    for path in [
        data::get_v1_movies_raw(),
        data::get_v1_movies_with_settings(),
        data::get_v1_rubygems_with_settings(),
    ] {
        let options = Opt {
            import_dump: Some(path.into()),
            ..default_settings(temp.path())
        };
        let error = Server::new_with_options(options)
            .await
            .map(|_| ())
            .unwrap_err();

        assert_eq!(error.to_string(), "The version 1 of the dumps is not supported anymore. You can re-export your dump from a version between 0.21 and 0.24, or start fresh from a version 0.25 onwards.");
    }
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v2_movie_raw() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v2_movies_raw()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("indexUID"));
    assert_eq!(indexes[0]["name"], json!("indexUID"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("indexUID");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"genres": 53, "id": 53, "overview": 53, "poster": 53, "release_date": 53, "title": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({"displayedAttributes": ["*"], "searchableAttributes": ["*"], "filterableAttributes": [], "sortableAttributes": [], "rankingRules": ["words", "typo", "proximity", "attribute", "exactness"], "stopWords": [], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": {"oneTypo": 5, "twoTypos": 9}, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks,
        json!({ "results": [{"uid": 0, "indexUid": "indexUID", "status": "succeeded", "type": "documentAddition", "details": { "receivedDocuments": 0, "indexedDocuments": 31944 }, "duration": "PT41.751156S", "enqueuedAt": "2021-09-08T08:30:30.550282Z", "startedAt": "2021-09-08T08:30:30.553012Z", "finishedAt": "2021-09-08T08:31:12.304168Z"}]})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(100, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"id": 100, "title": "Lock, Stock and Two Smoking Barrels", "overview": "A card shark and his unwillingly-enlisted friends need to make a lot of cash quick after losing a sketchy poker match. To do this they decide to pull a heist on a small-time gang who happen to be operating out of the flat next door.", "genres": ["Comedy", "Crime"], "poster": "https://image.tmdb.org/t/p/w500/8kSerJrhrJWKLk1LViesGcnrUPE.jpg", "release_date": 889056000})
    );

    let (document, code) = index.get_document(500, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"id": 500, "title": "Reservoir Dogs", "overview": "A botched robbery indicates a police informant, and the pressure mounts in the aftermath at a warehouse. Crime begets violence as the survivors -- veteran Mr. White, newcomer Mr. Orange, psychopathic parolee Mr. Blonde, bickering weasel Mr. Pink and Nice Guy Eddie -- unravel.", "genres": ["Crime", "Thriller"], "poster": "https://image.tmdb.org/t/p/w500/AjTtJNumZyUDz33VtMlF1K8JPsE.jpg", "release_date": 715392000})
    );

    let (document, code) = index.get_document(10006, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"id": 10006, "title": "Wild Seven", "overview": "In this darkly karmic vision of Arizona, a man who breathes nothing but ill will begins a noxious domino effect as quickly as an uncontrollable virus kills. As he exits Arizona State Penn after twenty-one long years, Wilson has only one thing on the brain, leveling the score with career criminal, Mackey Willis.", "genres": ["Action", "Crime", "Drama"], "poster": "https://image.tmdb.org/t/p/w500/y114dTPoqn8k2Txps4P2tI95YCS.jpg", "release_date": 1136073600})
    );
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v2_movie_with_settings() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v2_movies_with_settings()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("indexUID"));
    assert_eq!(indexes[0]["name"], json!("indexUID"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("indexUID");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"genres": 53, "id": 53, "overview": 53, "poster": 53, "release_date": 53, "title": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({ "displayedAttributes": ["title", "genres", "overview", "poster", "release_date"], "searchableAttributes": ["title", "overview"], "filterableAttributes": ["genres"], "sortableAttributes": [], "rankingRules": ["words", "typo", "proximity", "attribute", "exactness"], "stopWords": ["of", "the"], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": { "oneTypo": 5, "twoTypos": 9 }, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks,
        json!({ "results": [{ "uid": 1, "indexUid": "indexUID", "status": "succeeded", "type": "settingsUpdate", "details": { "displayedAttributes": ["title", "genres", "overview", "poster", "release_date"], "searchableAttributes": ["title", "overview"], "filterableAttributes": ["genres"], "stopWords": ["of", "the"] }, "duration": "PT37.488777S", "enqueuedAt": "2021-09-08T08:24:02.323444Z", "startedAt": "2021-09-08T08:24:02.324145Z", "finishedAt": "2021-09-08T08:24:39.812922Z" }, { "uid": 0, "indexUid": "indexUID", "status": "succeeded", "type": "documentAddition", "details": { "receivedDocuments": 0, "indexedDocuments": 31944 }, "duration": "PT39.941318S", "enqueuedAt": "2021-09-08T08:21:14.742672Z", "startedAt": "2021-09-08T08:21:14.750166Z", "finishedAt": "2021-09-08T08:21:54.691484Z" }]})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(100, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "title": "Lock, Stock and Two Smoking Barrels", "genres": ["Comedy", "Crime"], "overview": "A card shark and his unwillingly-enlisted friends need to make a lot of cash quick after losing a sketchy poker match. To do this they decide to pull a heist on a small-time gang who happen to be operating out of the flat next door.", "poster": "https://image.tmdb.org/t/p/w500/8kSerJrhrJWKLk1LViesGcnrUPE.jpg", "release_date": 889056000 })
    );

    let (document, code) = index.get_document(500, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"title": "Reservoir Dogs", "genres": ["Crime", "Thriller"], "overview": "A botched robbery indicates a police informant, and the pressure mounts in the aftermath at a warehouse. Crime begets violence as the survivors -- veteran Mr. White, newcomer Mr. Orange, psychopathic parolee Mr. Blonde, bickering weasel Mr. Pink and Nice Guy Eddie -- unravel.", "poster": "https://image.tmdb.org/t/p/w500/AjTtJNumZyUDz33VtMlF1K8JPsE.jpg", "release_date": 715392000})
    );

    let (document, code) = index.get_document(10006, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"title": "Wild Seven", "genres": ["Action", "Crime", "Drama"], "overview": "In this darkly karmic vision of Arizona, a man who breathes nothing but ill will begins a noxious domino effect as quickly as an uncontrollable virus kills. As he exits Arizona State Penn after twenty-one long years, Wilson has only one thing on the brain, leveling the score with career criminal, Mackey Willis.", "poster": "https://image.tmdb.org/t/p/w500/y114dTPoqn8k2Txps4P2tI95YCS.jpg", "release_date": 1136073600})
    );
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v2_rubygems_with_settings() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v2_rubygems_with_settings()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("rubygems"));
    assert_eq!(indexes[0]["name"], json!("rubygems"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("rubygems");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"description": 53, "id": 53, "name": 53, "summary": 53, "total_downloads": 53, "version": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({"displayedAttributes": ["name", "summary", "description", "version", "total_downloads"], "searchableAttributes": ["name", "summary"], "filterableAttributes": ["version"], "sortableAttributes": [], "rankingRules": ["typo", "words", "fame:desc", "proximity", "attribute", "exactness", "total_downloads:desc"], "stopWords": [], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": {"oneTypo": 5, "twoTypos": 9}, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks["results"][0],
        json!({"uid": 92, "indexUid": "rubygems", "status": "succeeded", "type": "documentAddition", "details": {"receivedDocuments": 0, "indexedDocuments": 1042}, "duration": "PT14.034672S", "enqueuedAt": "2021-09-08T08:40:31.390775Z", "startedAt": "2021-09-08T08:51:39.060642Z", "finishedAt": "2021-09-08T08:51:53.095314Z"})
    );
    assert_eq!(
        tasks["results"][92],
        json!({"uid": 0, "indexUid": "rubygems", "status": "succeeded", "type": "settingsUpdate", "details": {"displayedAttributes": ["name", "summary", "description", "version", "total_downloads"], "searchableAttributes": ["name", "summary"], "filterableAttributes": ["version"], "rankingRules": ["typo", "words", "desc(fame)", "proximity", "attribute", "exactness", "desc(total_downloads)"]}, "duration": "PT0.008886S", "enqueuedAt": "2021-09-08T08:40:28.660188Z", "startedAt": "2021-09-08T08:40:28.660766Z", "finishedAt": "2021-09-08T08:40:28.669652Z"})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(188040, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"name": "meilisearch", "summary": "An easy-to-use ruby client for Meilisearch API", "description": "An easy-to-use ruby client for Meilisearch API. See https://github.com/meilisearch/MeiliSearch", "version": "0.15.2", "total_downloads": "7465"})
    );

    let (document, code) = index.get_document(191940, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"name": "doggo", "summary": "RSpec 3 formatter - documentation, with progress indication", "description": "Similar to \"rspec -f d\", but also indicates progress by showing the current test number and total test count on each line.", "version": "1.1.0", "total_downloads": "9394"})
    );

    let (document, code) = index.get_document(159227, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"name": "vortex-of-agony", "summary": "You dont need to use nodejs or go, just install this plugin. It will crash your application at random", "description": "You dont need to use nodejs or go, just install this plugin. It will crash your application at random", "version": "0.1.0", "total_downloads": "1007"})
    );
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v3_movie_raw() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v3_movies_raw()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("indexUID"));
    assert_eq!(indexes[0]["name"], json!("indexUID"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("indexUID");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"genres": 53, "id": 53, "overview": 53, "poster": 53, "release_date": 53, "title": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({"displayedAttributes": ["*"], "searchableAttributes": ["*"], "filterableAttributes": [], "sortableAttributes": [], "rankingRules": ["words", "typo", "proximity", "attribute", "exactness"], "stopWords": [], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": {"oneTypo": 5, "twoTypos": 9}, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks,
        json!({ "results": [{"uid": 0, "indexUid": "indexUID", "status": "succeeded", "type": "documentAddition", "details": { "receivedDocuments": 0, "indexedDocuments": 31944 }, "duration": "PT41.751156S", "enqueuedAt": "2021-09-08T08:30:30.550282Z", "startedAt": "2021-09-08T08:30:30.553012Z", "finishedAt": "2021-09-08T08:31:12.304168Z"}]})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(100, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"id": 100, "title": "Lock, Stock and Two Smoking Barrels", "overview": "A card shark and his unwillingly-enlisted friends need to make a lot of cash quick after losing a sketchy poker match. To do this they decide to pull a heist on a small-time gang who happen to be operating out of the flat next door.", "genres": ["Comedy", "Crime"], "poster": "https://image.tmdb.org/t/p/w500/8kSerJrhrJWKLk1LViesGcnrUPE.jpg", "release_date": 889056000})
    );

    let (document, code) = index.get_document(500, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"id": 500, "title": "Reservoir Dogs", "overview": "A botched robbery indicates a police informant, and the pressure mounts in the aftermath at a warehouse. Crime begets violence as the survivors -- veteran Mr. White, newcomer Mr. Orange, psychopathic parolee Mr. Blonde, bickering weasel Mr. Pink and Nice Guy Eddie -- unravel.", "genres": ["Crime", "Thriller"], "poster": "https://image.tmdb.org/t/p/w500/AjTtJNumZyUDz33VtMlF1K8JPsE.jpg", "release_date": 715392000})
    );

    let (document, code) = index.get_document(10006, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"id": 10006, "title": "Wild Seven", "overview": "In this darkly karmic vision of Arizona, a man who breathes nothing but ill will begins a noxious domino effect as quickly as an uncontrollable virus kills. As he exits Arizona State Penn after twenty-one long years, Wilson has only one thing on the brain, leveling the score with career criminal, Mackey Willis.", "genres": ["Action", "Crime", "Drama"], "poster": "https://image.tmdb.org/t/p/w500/y114dTPoqn8k2Txps4P2tI95YCS.jpg", "release_date": 1136073600})
    );
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v3_movie_with_settings() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v3_movies_with_settings()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("indexUID"));
    assert_eq!(indexes[0]["name"], json!("indexUID"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("indexUID");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"genres": 53, "id": 53, "overview": 53, "poster": 53, "release_date": 53, "title": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({ "displayedAttributes": ["title", "genres", "overview", "poster", "release_date"], "searchableAttributes": ["title", "overview"], "filterableAttributes": ["genres"], "sortableAttributes": [], "rankingRules": ["words", "typo", "proximity", "attribute", "exactness"], "stopWords": ["of", "the"], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": { "oneTypo": 5, "twoTypos": 9 }, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks,
        json!({ "results": [{ "uid": 1, "indexUid": "indexUID", "status": "succeeded", "type": "settingsUpdate", "details": { "displayedAttributes": ["title", "genres", "overview", "poster", "release_date"], "searchableAttributes": ["title", "overview"], "filterableAttributes": ["genres"], "stopWords": ["of", "the"] }, "duration": "PT37.488777S", "enqueuedAt": "2021-09-08T08:24:02.323444Z", "startedAt": "2021-09-08T08:24:02.324145Z", "finishedAt": "2021-09-08T08:24:39.812922Z" }, { "uid": 0, "indexUid": "indexUID", "status": "succeeded", "type": "documentAddition", "details": { "receivedDocuments": 0, "indexedDocuments": 31944 }, "duration": "PT39.941318S", "enqueuedAt": "2021-09-08T08:21:14.742672Z", "startedAt": "2021-09-08T08:21:14.750166Z", "finishedAt": "2021-09-08T08:21:54.691484Z" }]})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(100, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "title": "Lock, Stock and Two Smoking Barrels", "genres": ["Comedy", "Crime"], "overview": "A card shark and his unwillingly-enlisted friends need to make a lot of cash quick after losing a sketchy poker match. To do this they decide to pull a heist on a small-time gang who happen to be operating out of the flat next door.", "poster": "https://image.tmdb.org/t/p/w500/8kSerJrhrJWKLk1LViesGcnrUPE.jpg", "release_date": 889056000 })
    );

    let (document, code) = index.get_document(500, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"title": "Reservoir Dogs", "genres": ["Crime", "Thriller"], "overview": "A botched robbery indicates a police informant, and the pressure mounts in the aftermath at a warehouse. Crime begets violence as the survivors -- veteran Mr. White, newcomer Mr. Orange, psychopathic parolee Mr. Blonde, bickering weasel Mr. Pink and Nice Guy Eddie -- unravel.", "poster": "https://image.tmdb.org/t/p/w500/AjTtJNumZyUDz33VtMlF1K8JPsE.jpg", "release_date": 715392000})
    );

    let (document, code) = index.get_document(10006, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"title": "Wild Seven", "genres": ["Action", "Crime", "Drama"], "overview": "In this darkly karmic vision of Arizona, a man who breathes nothing but ill will begins a noxious domino effect as quickly as an uncontrollable virus kills. As he exits Arizona State Penn after twenty-one long years, Wilson has only one thing on the brain, leveling the score with career criminal, Mackey Willis.", "poster": "https://image.tmdb.org/t/p/w500/y114dTPoqn8k2Txps4P2tI95YCS.jpg", "release_date": 1136073600})
    );
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v3_rubygems_with_settings() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v3_rubygems_with_settings()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("rubygems"));
    assert_eq!(indexes[0]["name"], json!("rubygems"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("rubygems");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"description": 53, "id": 53, "name": 53, "summary": 53, "total_downloads": 53, "version": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({"displayedAttributes": ["name", "summary", "description", "version", "total_downloads"], "searchableAttributes": ["name", "summary"], "filterableAttributes": ["version"], "sortableAttributes": [], "rankingRules": ["typo", "words", "fame:desc", "proximity", "attribute", "exactness", "total_downloads:desc"], "stopWords": [], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": {"oneTypo": 5, "twoTypos": 9}, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks["results"][0],
        json!({"uid": 92, "indexUid": "rubygems", "status": "succeeded", "type": "documentAddition", "details": {"receivedDocuments": 0, "indexedDocuments": 1042}, "duration": "PT14.034672S", "enqueuedAt": "2021-09-08T08:40:31.390775Z", "startedAt": "2021-09-08T08:51:39.060642Z", "finishedAt": "2021-09-08T08:51:53.095314Z"})
    );
    assert_eq!(
        tasks["results"][92],
        json!({"uid": 0, "indexUid": "rubygems", "status": "succeeded", "type": "settingsUpdate", "details": {"displayedAttributes": ["name", "summary", "description", "version", "total_downloads"], "searchableAttributes": ["name", "summary"], "filterableAttributes": ["version"], "rankingRules": ["typo", "words", "desc(fame)", "proximity", "attribute", "exactness", "desc(total_downloads)"]}, "duration": "PT0.008886S", "enqueuedAt": "2021-09-08T08:40:28.660188Z", "startedAt": "2021-09-08T08:40:28.660766Z", "finishedAt": "2021-09-08T08:40:28.669652Z"})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(188040, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"name": "meilisearch", "summary": "An easy-to-use ruby client for Meilisearch API", "description": "An easy-to-use ruby client for Meilisearch API. See https://github.com/meilisearch/MeiliSearch", "version": "0.15.2", "total_downloads": "7465"})
    );

    let (document, code) = index.get_document(191940, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"name": "doggo", "summary": "RSpec 3 formatter - documentation, with progress indication", "description": "Similar to \"rspec -f d\", but also indicates progress by showing the current test number and total test count on each line.", "version": "1.1.0", "total_downloads": "9394"})
    );

    let (document, code) = index.get_document(159227, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({"name": "vortex-of-agony", "summary": "You dont need to use nodejs or go, just install this plugin. It will crash your application at random", "description": "You dont need to use nodejs or go, just install this plugin. It will crash your application at random", "version": "0.1.0", "total_downloads": "1007"})
    );
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v4_movie_raw() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v4_movies_raw()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("indexUID"));
    assert_eq!(indexes[0]["name"], json!("indexUID"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("indexUID");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"genres": 53, "id": 53, "overview": 53, "poster": 53, "release_date": 53, "title": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({ "displayedAttributes": ["*"], "searchableAttributes": ["*"], "filterableAttributes": [], "sortableAttributes": [], "rankingRules": ["words", "typo", "proximity", "attribute", "exactness"], "stopWords": [], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": {"oneTypo": 5, "twoTypos": 9}, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks,
        json!({ "results": [{"uid": 0, "indexUid": "indexUID", "status": "succeeded", "type": "documentAddition", "details": { "receivedDocuments": 0, "indexedDocuments": 31944 }, "duration": "PT41.751156S", "enqueuedAt": "2021-09-08T08:30:30.550282Z", "startedAt": "2021-09-08T08:30:30.553012Z", "finishedAt": "2021-09-08T08:31:12.304168Z"}]})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(100, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "id": 100, "title": "Lock, Stock and Two Smoking Barrels", "overview": "A card shark and his unwillingly-enlisted friends need to make a lot of cash quick after losing a sketchy poker match. To do this they decide to pull a heist on a small-time gang who happen to be operating out of the flat next door.", "genres": ["Comedy", "Crime"], "poster": "https://image.tmdb.org/t/p/w500/8kSerJrhrJWKLk1LViesGcnrUPE.jpg", "release_date": 889056000})
    );

    let (document, code) = index.get_document(500, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "id": 500, "title": "Reservoir Dogs", "overview": "A botched robbery indicates a police informant, and the pressure mounts in the aftermath at a warehouse. Crime begets violence as the survivors -- veteran Mr. White, newcomer Mr. Orange, psychopathic parolee Mr. Blonde, bickering weasel Mr. Pink and Nice Guy Eddie -- unravel.", "genres": ["Crime", "Thriller"], "poster": "https://image.tmdb.org/t/p/w500/AjTtJNumZyUDz33VtMlF1K8JPsE.jpg", "release_date": 715392000})
    );

    let (document, code) = index.get_document(10006, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "id": 10006, "title": "Wild Seven", "overview": "In this darkly karmic vision of Arizona, a man who breathes nothing but ill will begins a noxious domino effect as quickly as an uncontrollable virus kills. As he exits Arizona State Penn after twenty-one long years, Wilson has only one thing on the brain, leveling the score with career criminal, Mackey Willis.", "genres": ["Action", "Crime", "Drama"], "poster": "https://image.tmdb.org/t/p/w500/y114dTPoqn8k2Txps4P2tI95YCS.jpg", "release_date": 1136073600})
    );
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v4_movie_with_settings() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v4_movies_with_settings()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("indexUID"));
    assert_eq!(indexes[0]["name"], json!("indexUID"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("indexUID");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"genres": 53, "id": 53, "overview": 53, "poster": 53, "release_date": 53, "title": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({ "displayedAttributes": ["title", "genres", "overview", "poster", "release_date"], "searchableAttributes": ["title", "overview"], "filterableAttributes": ["genres"], "sortableAttributes": [], "rankingRules": ["words", "typo", "proximity", "attribute", "exactness"], "stopWords": ["of", "the"], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": { "oneTypo": 5, "twoTypos": 9 }, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks,
        json!({ "results": [{ "uid": 1, "indexUid": "indexUID", "status": "succeeded", "type": "settingsUpdate", "details": { "displayedAttributes": ["title", "genres", "overview", "poster", "release_date"], "searchableAttributes": ["title", "overview"], "filterableAttributes": ["genres"], "stopWords": ["of", "the"] }, "duration": "PT37.488777S", "enqueuedAt": "2021-09-08T08:24:02.323444Z", "startedAt": "2021-09-08T08:24:02.324145Z", "finishedAt": "2021-09-08T08:24:39.812922Z" }, { "uid": 0, "indexUid": "indexUID", "status": "succeeded", "type": "documentAddition", "details": { "receivedDocuments": 0, "indexedDocuments": 31944 }, "duration": "PT39.941318S", "enqueuedAt": "2021-09-08T08:21:14.742672Z", "startedAt": "2021-09-08T08:21:14.750166Z", "finishedAt": "2021-09-08T08:21:54.691484Z" }]})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(100, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "title": "Lock, Stock and Two Smoking Barrels", "genres": ["Comedy", "Crime"], "overview": "A card shark and his unwillingly-enlisted friends need to make a lot of cash quick after losing a sketchy poker match. To do this they decide to pull a heist on a small-time gang who happen to be operating out of the flat next door.", "poster": "https://image.tmdb.org/t/p/w500/8kSerJrhrJWKLk1LViesGcnrUPE.jpg", "release_date": 889056000 })
    );

    let (document, code) = index.get_document(500, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "title": "Reservoir Dogs", "genres": ["Crime", "Thriller"], "overview": "A botched robbery indicates a police informant, and the pressure mounts in the aftermath at a warehouse. Crime begets violence as the survivors -- veteran Mr. White, newcomer Mr. Orange, psychopathic parolee Mr. Blonde, bickering weasel Mr. Pink and Nice Guy Eddie -- unravel.", "poster": "https://image.tmdb.org/t/p/w500/AjTtJNumZyUDz33VtMlF1K8JPsE.jpg", "release_date": 715392000})
    );

    let (document, code) = index.get_document(10006, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "title": "Wild Seven", "genres": ["Action", "Crime", "Drama"], "overview": "In this darkly karmic vision of Arizona, a man who breathes nothing but ill will begins a noxious domino effect as quickly as an uncontrollable virus kills. As he exits Arizona State Penn after twenty-one long years, Wilson has only one thing on the brain, leveling the score with career criminal, Mackey Willis.", "poster": "https://image.tmdb.org/t/p/w500/y114dTPoqn8k2Txps4P2tI95YCS.jpg", "release_date": 1136073600})
    );
}

#[actix_rt::test]
#[cfg_attr(target_os = "windows", ignore)]
async fn import_dump_v4_rubygems_with_settings() {
    let temp = tempfile::tempdir().unwrap();

    let options = Opt {
        import_dump: Some(data::get_v4_rubygems_with_settings()),
        ..default_settings(temp.path())
    };
    let server = Server::new_with_options(options).await.unwrap();

    let (indexes, code) = server.list_indexes().await;
    assert_eq!(code, 200);

    assert_eq!(indexes.as_array().unwrap().len(), 1);
    assert_eq!(indexes[0]["uid"], json!("rubygems"));
    assert_eq!(indexes[0]["name"], json!("rubygems"));
    assert_eq!(indexes[0]["primaryKey"], json!("id"));

    let index = server.index("rubygems");

    let (stats, code) = index.stats().await;
    assert_eq!(code, 200);
    assert_eq!(
        stats,
        json!({ "numberOfDocuments": 53, "isIndexing": false, "fieldDistribution": {"description": 53, "id": 53, "name": 53, "summary": 53, "total_downloads": 53, "version": 53 }})
    );

    let (settings, code) = index.settings().await;
    assert_eq!(code, 200);
    assert_eq!(
        settings,
        json!({ "displayedAttributes": ["name", "summary", "description", "version", "total_downloads"], "searchableAttributes": ["name", "summary"], "filterableAttributes": ["version"], "sortableAttributes": [], "rankingRules": ["typo", "words", "fame:desc", "proximity", "attribute", "exactness", "total_downloads:desc"], "stopWords": [], "synonyms": {}, "distinctAttribute": null, "typo": {"enabled": true, "minWordLengthForTypo": {"oneTypo": 5, "twoTypos": 9}, "disableOnWords": [], "disableOnAttributes": [] }})
    );

    let (tasks, code) = index.list_tasks().await;
    assert_eq!(code, 200);
    assert_eq!(
        tasks["results"][0],
        json!({ "uid": 92, "indexUid": "rubygems", "status": "succeeded", "type": "documentAddition", "details": {"receivedDocuments": 0, "indexedDocuments": 1042}, "duration": "PT14.034672S", "enqueuedAt": "2021-09-08T08:40:31.390775Z", "startedAt": "2021-09-08T08:51:39.060642Z", "finishedAt": "2021-09-08T08:51:53.095314Z"})
    );
    assert_eq!(
        tasks["results"][92],
        json!({ "uid": 0, "indexUid": "rubygems", "status": "succeeded", "type": "settingsUpdate", "details": {"displayedAttributes": ["name", "summary", "description", "version", "total_downloads"], "searchableAttributes": ["name", "summary"], "filterableAttributes": ["version"], "rankingRules": ["typo", "words", "desc(fame)", "proximity", "attribute", "exactness", "desc(total_downloads)"]}, "duration": "PT0.008886S", "enqueuedAt": "2021-09-08T08:40:28.660188Z", "startedAt": "2021-09-08T08:40:28.660766Z", "finishedAt": "2021-09-08T08:40:28.669652Z"})
    );

    // finally we're just going to check that we can still get a few documents by id
    let (document, code) = index.get_document(188040, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "name": "meilisearch", "summary": "An easy-to-use ruby client for Meilisearch API", "description": "An easy-to-use ruby client for Meilisearch API. See https://github.com/meilisearch/MeiliSearch", "version": "0.15.2", "total_downloads": "7465"})
    );

    let (document, code) = index.get_document(191940, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "name": "doggo", "summary": "RSpec 3 formatter - documentation, with progress indication", "description": "Similar to \"rspec -f d\", but also indicates progress by showing the current test number and total test count on each line.", "version": "1.1.0", "total_downloads": "9394"})
    );

    let (document, code) = index.get_document(159227, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        document,
        json!({ "name": "vortex-of-agony", "summary": "You dont need to use nodejs or go, just install this plugin. It will crash your application at random", "description": "You dont need to use nodejs or go, just install this plugin. It will crash your application at random", "version": "0.1.0", "total_downloads": "1007"})
    );
}
