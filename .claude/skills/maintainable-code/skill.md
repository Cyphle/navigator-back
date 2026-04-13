# Create Maintainable Software with Tests and Well Naming

## Short description (for LLMs)
Guidelines for writing maintainable Rust code in navigator-back: naming conventions per layer, short and readable tests using factory helpers, avoiding test boilerplate, and keeping each layer focused on its single responsibility.

## Persona
Tu es un ingénieur backend Rust qui valorise le code qui se lit comme une documentation. Tu écris des tests qui communiquent clairement l'intention (Given / When / Then), tu nommes les choses au bon niveau d'abstraction pour leur couche, et tu résistes à la tentation d'ajouter des abstractions qui ne sont pas encore nécessaires. Tu gardes des tests le plus court possible pour favoriser la lisibilité quitte à les couper en plusieurs.

## Quand utiliser ce skill
- Lors de l'écriture de tout nouveau code dans ce projet
- Lors de la revue ou du refactoring de code existant pour plus de clarté
- Comme référence avant de soumettre une fonctionnalité
- Lors de la modification de code

## Le skill en détail

### 1. Conventions de nommage par couche

| Couche | Pattern de nommage | Exemple |
|---|---|---|
| Modèle domaine | nom, sans suffixe | `Family`, `BankAccount`, `MagicListItem` |
| Command (entrée du use case) | `Create<Noun>Command`, `Update<Noun>Command` | `CreateFamilyCommand` |
| Trait repository | `<Noun>Repository` | `FamilyRepository` |
| Impl SQLx | `Sqlx<Noun>Repository` | `SqlxFamilyRepository` |
| Entité (ligne DB) | `<Noun>Entity` | `FamilyEntity` |
| Fonction use case | `<verb>_<noun>_use_case` | `get_families_use_case` |
| Handler controller | `<verb>_<noun>_endpoint` | `get_families_endpoint` |
| Fonction middleware | `<verb>_<noun>_middleware` | `get_families_middleware` |
| Struct requête HTTP | `<Verb><Noun>Request` | `CreateFamilyRequest` |
| Vue / réponse HTTP | `<Noun>View`, `<Noun>OverviewView` | `FamilyView`, `BankAccountOverviewView` |
| Repository mock de test | `Mock<Noun>Repository` | `MockFamilyRepository` |
| Factory de requête mock | `Mock<Noun>Request` | `MockFamilyRequest` |

### 2. Nommage des tests
Les noms de fonctions de test décrivent ce que le système fait, pas comment :
```rust
// Bien — décrit le comportement observable
async fn should_return_families_for_authenticated_user()
async fn should_error_on_db_connection_failure()
async fn should_call_get_families_application_layer()

// Mal — décrit l'implémentation
async fn test_get()
async fn family_repository_get_families()
```
Utiliser la structure en trois blocs Given / When / Then avec commentaires :
```rust
#[actix_web::test]
async fn should_return_families() {
    // Given
    let state = make_state_ok();

    // When
    let result = get_families_use_case(state, "john".to_string()).await;

    // Then
    let families = result.expect("families");
    assert_eq!(families.len(), 2);
}
```

### 3. Helpers factory plutôt que boilerplate inline
Extraire la mise en place du state dans de petites fonctions factory nommées en haut du module de test. Garder les factories minimales — configurer uniquement ce dont le test se préoccupe réellement :
```rust
fn make_state_ok() -> web::Data<MockActixState> {
    mock_actix_state(
        MockPoolPostgres,
        MockStateConfig {
            families: Some(vec![Family { id: 1, name: "A".to_string(), ... }]),
            ..MockStateConfig::default()
        },
    )
}

fn make_state_repo_error() -> web::Data<MockActixState> {
    mock_actix_state(
        MockPoolPostgres,
        MockStateConfig { family_should_error: true, ..MockStateConfig::default() },
    )
}
```
Utiliser des factories de requêtes style builder pour les données de body HTTP :
```rust
let request = MockFamilyRequest::new("My family".to_string())
    .add_creator_relation("PARENT".to_string())
    .add_member(CreateFamilyMemberRequest { ... })
    .build();
```

### 4. Couverture de test par couche
Chaque couche a une responsabilité différente et se teste donc différemment :

**Tests de use case** — vérifier que les bonnes données sont retournées et le bon type d'erreur sur échec :
```
- should_return_<result>_on_success
- should_error_on_db_connection_failure
- should_error_on_repository_failure
```

**Tests de middleware** — vérifier que la bonne fonction use case est appelée (pas les données) :
```
- should_call_<use_case>_application_layer
```
Utiliser `spy!` du crate `spy` pour vérifier le nombre d'appels.

**Tests d'intégration de repository** — vérifier la correction du SQL via `#[sqlx_testcontainers::test]` :
```
- test_get_<noun>_for
- test_create_<noun>
- test_<noun>_not_found_returns_error
```

**Tests de struct de requête** — vérifier la désérialisation JSON :
```
- should_deserialize_create_<noun>_request
```

### 5. Responsabilités par couche — ne pas les croiser

| Couche | Possède | Ne fait jamais |
|---|---|---|
| Modèle domaine | Forme des données, enums domaine | Base de données, HTTP |
| Trait repository | Contrat d'accès aux données | Logique métier |
| Impl repository | Requêtes SQL, mapping de lignes | Transactions, erreurs au-delà de `sqlx::Error` |
| Use case | Cycle de vie de la transaction, logique métier, mapping d'erreurs | HTTP, SQL brut |
| Middleware | Extraction de session, mapping de réponse HTTP, construction de command | Logique métier |
| Controller | Déclaration de route, délégation au middleware | Toute logique |

### 6. Règles de gestion des erreurs
- Les méthodes de repository retournent `sqlx::Error`.
- Les use cases mappent toutes les erreurs vers `Box<dyn ApplicationError>` via `RepositoryError`.
- Les middlewares loguent les erreurs avec `error!(...)` et retournent `HttpResponse::InternalServerError()`.
- Ne jamais `.unwrap()` dans les chemins de code de production — seulement dans les tests.

### 7. Erreurs courantes à éviter
- **Ne pas démarrer la transaction dans le repository** — c'est le use case qui l'owner.
- **Ne pas retourner des types domaine depuis les handlers HTTP** — toujours mapper vers une struct de vue avec `#[derive(Serialize)]`.
- **Ne pas mocker les repositories pour les tests d'intégration** — utiliser `#[sqlx_testcontainers::test]`.
- **Ne pas utiliser `.unwrap()` dans le code de production** — seulement dans les blocs d'arrange/assert des tests.
- **Ne pas ajouter `pub` à des champs/méthodes qui n'en ont pas besoin** — notamment les champs d'entités.
- **Ne pas tester la même chose dans plusieurs couches** — faire confiance à la couche en dessous, espionner la couche au-dessus.

### 8. Checklist avant de finir toute fonctionnalité
- [ ] Nommage suit le tableau de conventions ci-dessus
- [ ] Chaque test a une structure claire Given / When / Then
- [ ] Helpers factory utilisés plutôt que du boilerplate inline répété
- [ ] Les trois scénarios de test couverts pour les use cases (ok, db error, repo error)
- [ ] Tests middleware utilisent `spy!` — pas d'assertions sur les données
- [ ] Tests repository utilisent `#[sqlx_testcontainers::test]`
- [ ] Aucun `.unwrap()` dans le code de production
- [ ] Aucune logique métier sous la couche use case
- [ ] Aucune préoccupation HTTP/SQL au-dessus de la couche middleware/repository
- [ ] `cargo test` passe
