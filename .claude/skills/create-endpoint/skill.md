# Create an Endpoint with its Middleware

## Short description (for LLMs)
Creates a thin Actix-Web controller + middleware pair for a new HTTP endpoint following the navigator-back pattern: controller delegates immediately to middleware, middleware handles session extraction and HTTP response mapping, tests use `spy!` to assert the use case is called. All endpoints must comply with REST level 2 (resources as nouns in URLs, HTTP verbs for actions, meaningful status codes).

## Persona
Tu es un ingénieur backend Rust travaillant sur navigator-back. Tu gardes les controllers aussi fins que possible (2–3 lignes de code) et tu places toute la logique de la couche HTTP — lecture de session, mapping de réponse, gestion d'erreurs — dans la fonction middleware. Tu ne mets jamais de logique métier dans ces deux couches.

## Quand utiliser ce skill
- Un nouveau chemin URL doit être exposé
- Tu as déjà créé ou identifié le use case à appeler
- Tu dois ajouter des fichiers controller + middleware dans `src/domains/<domain>/http/`

## Le skill en détail

### 1. Emplacements des fichiers
```
src/domains/<domain>/http/<domain>_controller.rs   # route handlers
src/domains/<domain>/http/<domain>_middleware.rs   # session + mapping réponse
src/domains/<domain>/http/<domain>_requests.rs     # structs de requête JSON
src/domains/<domain>/http/<domain>_views.rs        # structs de réponse JSON
```
Les exposer dans `src/domains/<domain>/http/mod.rs`.

### 2. REST niveau 2 — règles obligatoires

**URLs : noms de ressources, jamais de verbes**
```
// Bien
GET    /families              # liste
GET    /families/{id}         # détail
POST   /families              # création
PUT    /families/{id}         # remplacement complet
PATCH  /families/{id}         # mise à jour partielle
DELETE /families/{id}         # suppression

// Mal
GET  /getFamilies
POST /createFamily
POST /families/delete
```

**Verbes HTTP et codes de statut correspondants**

| Action | Méthode | Succès | Erreur client | Erreur serveur |
|---|---|---|---|---|
| Lire une collection | `GET` | `200 Ok` | `401 Unauthorized` | `500 Internal Server Error` |
| Lire une ressource | `GET` | `200 Ok` | `404 Not Found` | `500 Internal Server Error` |
| Créer | `POST` | `201 Created` | `400 Bad Request` | `500 Internal Server Error` |
| Remplacer | `PUT` | `200 Ok` | `404 Not Found` | `500 Internal Server Error` |
| Modifier | `PATCH` | `200 Ok` | `404 Not Found` | `500 Internal Server Error` |
| Supprimer | `DELETE` | `204 No Content` | `404 Not Found` | `500 Internal Server Error` |

Dans le middleware, utiliser le code de statut sémantiquement correct :
```rust
// Création réussie
HttpResponse::Created().json(view)

// Suppression réussie
HttpResponse::NoContent().finish()

// Ressource non trouvée
HttpResponse::NotFound().json(e.get_message())

// Requête invalide (validation échouée)
HttpResponse::BadRequest().json(e.get_message())
```

**Ressources imbriquées** pour les relations parent/enfant :
```
GET  /families/{id}/members      # membres d'une famille
POST /families/{id}/members      # ajouter un membre
```

### 3. Controller (ultra-thin)
Le controller déclare uniquement la route et délègue au middleware :
```rust
#[get("/<route>")]
pub async fn <verb>_<noun>_endpoint(
    session: Session,
    state: web::Data<ActixState>,
) -> impl Responder {
    debug!("[Controller] <Verb> <noun>");
    <verb>_<noun>_middleware(session, state, <verb>_<noun>_use_case).await
}

// Pour les endpoints avec un body JSON :
#[post("/<route>")]
pub async fn create_<noun>_endpoint(
    payload: web::Json<Create<Noun>Request>,
    session: Session,
    state: web::Data<ActixState>,
) -> impl Responder {
    debug!("[Controller] Create <noun>");
    create_<noun>_middleware(session, state, payload.into_inner(), create_<noun>_use_case).await
}
```
Ne jamais mettre de `if`, `match` ou logique métier dans le controller.

### 3. Signature du middleware
Le use case est injecté comme paramètre générique de type fonction — c'est ce qui rend le middleware testable avec `spy!` :
```rust
pub async fn <verb>_<noun>_middleware<DB, UseCase, Fut>(
    session: Session,
    state: web::Data<ActixState<DB>>,
    use_case: UseCase,
) -> impl Responder
where
    DB: DbConnection,
    UseCase: Fn(web::Data<ActixState<DB>>, String) -> Fut,
    Fut: Future<Output = Result<OutputType, Box<dyn ApplicationError>>>,
```
Pour les endpoints avec un body, ajouter la requête entre `state` et `use_case` :
```rust
    request: Create<Noun>Request,
    use_case: UseCase, // Fn(state, username, command) -> Fut
```

### 4. Corps du middleware
```rust
pub async fn <verb>_<noun>_middleware<...>(...) -> impl Responder {
    debug!("[Middleware] <Verb> <noun>");

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError);

    match username {
        Ok(username) => match use_case(state, username /*, command */).await {
            Ok(result) => HttpResponse::Ok().json(<ViewType>::from(result)),
            Err(e) => {
                error!("Error <verb>ing <noun>: {:?}", e.get_message());
                HttpResponse::InternalServerError().json(e.get_message())
            }
        },
        Err(e) => {
            error!("Error getting username: {:?}", e.get_message());
            HttpResponse::InternalServerError().json(e.get_message())
        }
    }
}
```
Mapper les résultats domaine vers des structs de vue (`#[derive(Serialize)]`). Ne jamais retourner d'objets domaine bruts.

### 5. Structs de requête
```rust
#[derive(serde::Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Create<Noun>Request {
    pub field_one: String,
    pub field_two: bool,
}
```
Ajouter un test de désérialisation pour chaque struct de requête :
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn should_deserialize_create_<noun>_request() {
        let json = r#"{ "fieldOne": "value", "fieldTwo": true }"#;
        let req: Create<Noun>Request = serde_json::from_str(json).unwrap();
        assert_eq!(req.field_one, "value");
    }
}
```

### 6. Tests du middleware avec `spy!`
Chaque test de middleware vérifie que le bon use case est appelé (pas que les bonnes données sont retournées — c'est la responsabilité du use case) :
```rust
#[actix_web::test]
async fn should_call_<verb>_<noun>_application_layer() {
    // Given
    let state = mock_actix_state(MockPoolPostgres, MockStateConfig { ... });
    let (spy_handler, spy) = spy!();
    let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);

    // When
    let app = test::init_service(App::new().app_data(state.clone()).route(
        "/<route>",
        web::get().to({
            let spy_handler = Arc::clone(&spy_handler);
            move |session: actix_session::Session, state: web::Data<MockActixState>| {
                let spy_handler = Arc::clone(&spy_handler);
                async move {
                    session.insert("test_username", "mock_user").expect("set session");
                    super::<verb>_<noun>_middleware(session, state, move |state, username| {
                        (spy_handler)();
                        <verb>_<noun>_use_case(state, username)
                    })
                    .await
                }
            }
        }),
    ))
    .await;
    let req = test::TestRequest::get().uri("/<route>").to_request();
    let resp = test::call_service(&app, req).await;

    // Then
    assert_eq!(resp.status(), StatusCode::OK);
    drop(app);
    drop(spy_handler);
    assert_eq!(spy.snapshot().num_of_calls(), 1);
}
```

### 7. Enregistrer la route
Dans `main.rs` (ou le fichier de configuration des routes du domaine), ajouter :
```rust
.service(<verb>_<noun>_endpoint)
```

### 9. Checklist avant de finir
- [ ] URL en noms de ressources, aucun verbe dans le chemin
- [ ] Verbe HTTP correct pour l'action (GET / POST / PUT / PATCH / DELETE)
- [ ] Code de statut HTTP sémantiquement correct (201, 204, 404, 400…)
- [ ] Controller ≤ 5 lignes par endpoint, aucune logique
- [ ] Middleware générique sur `DB` et le use case `Fn`
- [ ] Extraction de session via `get_connected_username`
- [ ] Résultats domaine mappés vers des structs de vue avant sérialisation
- [ ] Struct de requête avec test de désérialisation
- [ ] Chaque middleware a un test `spy!` qui vérifie l'appel au use case
- [ ] Route enregistrée dans `main.rs`
- [ ] `cargo test` passe
