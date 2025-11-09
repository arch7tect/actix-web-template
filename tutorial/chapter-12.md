# Chapter 12: Web Page Handlers - Building the UI

## Overview

Web page handlers bridge your backend services with the frontend user interface, rendering server-side HTML using the Askama templates you created in Chapter 10. Unlike REST API handlers that return JSON, web handlers return complete HTML pages or HTML fragments, supporting both traditional form submissions and modern AJAX updates.

In Chapter 11, you created beautiful CSS styling for your interface. Now, in this chapter, you'll implement the handlers that bring those styles to life by connecting them to your backend. You'll display memos, create and edit them through forms, and update the UI dynamically without full page reloads. You'll learn the "Redirect After Post" pattern, progressive enhancement with vanilla JavaScript, and how to handle HTML form data in Actix Web.

By the end of this chapter, you'll see your polished CSS from Chapter 11 come alive in a fully functional web application that works without JavaScript (baseline experience) and enhances with JavaScript for smooth AJAX interactions.

## Prerequisites

### Completed Chapters
- Chapter 0: Prerequisites and Environment Setup
- Chapter 1: Core Application Setup
- Chapter 5: DTOs and Validation
- Chapter 7: Service Layer
- Chapter 10: Askama Templates
- Chapter 11: Static Assets and Styling

### Required Knowledge
- HTTP methods (GET, POST, PUT, DELETE, PATCH)
- HTML forms and form submission
- Basic JavaScript and Fetch API
- Understanding of server-side rendering

### System Requirements
- Working database setup from Chapter 2
- Templates from Chapter 10
- Running PostgreSQL instance

## Learning Objectives

By the end of this chapter, you will be able to:

1. Render Askama templates in HTTP handlers
2. Handle HTML form submissions (POST, PUT, DELETE)
3. Parse and validate form data in Actix Web
4. Implement full-page renders vs partial AJAX updates
5. Return appropriate HTML fragments for client-side updates
6. Use progressive enhancement for robust UX
7. Test web handlers with integration tests
8. Understand the trade-offs between SSR and CSR

## Concepts Covered

### Server-Side Rendering (SSR) in Actix Web

**Server-Side Rendering** generates complete HTML on the server and sends it to the browser. The client receives ready-to-display content:

```
Client Request → Actix Handler → Service → Repository → Database
                       ↓
                 Fetch Data
                       ↓
              Render Template (Askama)
                       ↓
              Complete HTML Response → Browser displays immediately
```

**Benefits**:
- **SEO-friendly**: Search engines see complete content
- **Fast initial render**: No waiting for JavaScript bundle
- **Works without JS**: Baseline experience for all users
- **Reduced client load**: Server does the rendering work

**Trade-offs**:
- **Full page reloads**: Traditional forms cause navigation
- **Server load**: Each render happens server-side
- **No client state**: State reset on navigation

**Solution**: Progressive enhancement - start with SSR, add JavaScript for smooth updates.

### Progressive Enhancement Pattern

**Progressive Enhancement** builds in layers:

1. **Base layer** (works for everyone): HTML + server-side rendering
2. **Enhancement layer** (better experience): JavaScript adds AJAX, avoiding page reloads
3. **Graceful degradation**: If JS fails or is disabled, site still works

Example in our app:

**Without JavaScript** (baseline):
```
User clicks "New Memo" → Full page navigation to /memos/new
User submits form → POST /memos → Redirect to homepage
```

**With JavaScript** (enhanced):
```
User clicks "New Memo" → Fetch form HTML → Show in modal
User submits form → AJAX POST /memos → Update memo list in-place
```

Both work. JavaScript just makes it smoother.

### HTML Form Handling in Actix Web

**Form encoding**: HTML forms send data as `application/x-www-form-urlencoded`:
```
title=Buy+milk&description=From+store&date_to=2025-01-15T10%3A00
```

**Actix Web extraction**:
```rust
use actix_web::web::Form;

#[post("/memos")]
pub async fn create_memo(
    form: Form<CreateMemoForm>  // Actix automatically deserializes
) -> Result<HttpResponse, AppError> {
    // form.title, form.description, etc.
}
```

**Form structs** with validation:
```rust
#[derive(Deserialize, Validate)]
pub struct CreateMemoForm {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub description: Option<String>,
    pub date_to: String,  // HTML sends as string, we parse to DateTime
}
```

**Validation happens before business logic**:
```rust
form.validate()
    .map_err(|e| AppError::Validation(format!("Validation failed: {}", e)))?;
```

### Full Page Renders vs Partial Updates

Our app uses **two rendering strategies**:

**Full page renders** (initial load):
```rust
// GET / → Complete HTML page
pub async fn index(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let memos = service.get_all_memos(params).await?;
    let template = IndexTemplate { memos: memos.data };
    Ok(HttpResponse::Ok().content_type("text/html").body(template.render()?))
}
```

Returns: `<!DOCTYPE html><html>...complete page...</html>`

**Partial updates** (AJAX):
```rust
// GET /web/memos → Just the memo list HTML fragment
pub async fn get_memos_list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let memos = service.get_all_memos(params).await?;
    let template = MemoListTemplate { memos: memos.data };
    Ok(HttpResponse::Ok().content_type("text/html").body(template.render()?))
}
```

Returns: `<div>...just the memo list...</div>` (fragment)

JavaScript replaces part of the page:
```javascript
fetch('/web/memos')
    .then(res => res.text())
    .then(html => {
        document.getElementById('memo-list').innerHTML = html;  // Swap HTML
    });
```

**Pattern**:
- Pages extend `base.html` (full documents)
- Components don't extend anything (fragments)
- Both use same service layer

### Redirect After Post Pattern

**Problem**: After POST/PUT/DELETE, what response should handlers return?

**Bad approach** (causes duplicate submissions):
```rust
#[post("/memos")]
pub async fn create_memo(...) -> Result<HttpResponse, AppError> {
    service.create_memo(dto).await?;
    // Return 200 OK with success page
    Ok(HttpResponse::Ok().body("Created!"))
}
// If user hits browser refresh → Form resubmits → Duplicate memo!
```

**Good approach** (Redirect After Post):
```rust
#[post("/memos")]
pub async fn create_memo(...) -> Result<HttpResponse, AppError> {
    service.create_memo(dto).await?;
    // Redirect to GET endpoint
    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", "/"))
        .finish())
}
// Browser follows redirect → Now at GET /
// User refresh → Just reloads page, doesn't resubmit form
```

**For AJAX**, return updated HTML instead:
```rust
#[post("/web/memos")]
pub async fn create_memo(...) -> Result<HttpResponse, AppError> {
    service.create_memo(dto).await?;

    // Return updated memo list for JavaScript to inject
    let memos = service.get_all_memos(params).await?;
    let template = MemoListTemplate { memos: memos.data };
    Ok(HttpResponse::Ok().body(template.render()?))
}
```

JavaScript updates the page in-place, no navigation needed.

### DateTime Parsing for HTML Forms

HTML `<input type="datetime-local">` sends format: `2025-01-15T10:30`

This isn't the same as `DateTime<Utc>`. We must parse it:

```rust
use chrono::{DateTime, Utc, NaiveDateTime};

// HTML form sends: "2025-01-15T10:30"
let date_to: DateTime<Utc> =
    NaiveDateTime::parse_from_str(&form.date_to, "%Y-%m-%dT%H:%M")
        .map_err(|_| AppError::Validation("Invalid date format".to_string()))?
        .and_utc();  // Convert to UTC
```

**Why NaiveDateTime**: HTML datetime-local has no timezone. We assume UTC for storage.

### Event Delegation for Dynamic Content

JavaScript that updates the DOM dynamically needs **event delegation**:

**Bad approach** (doesn't work for new elements):
```javascript
document.querySelectorAll('.btn-delete').forEach(btn => {
    btn.addEventListener('click', deleteHandler);
});
// New memos added via AJAX won't have listeners!
```

**Good approach** (event delegation):
```javascript
document.addEventListener('click', function(e) {
    if (e.target.dataset.action === 'delete') {
        deleteHandler(e.target.dataset.memoId);
    }
});
// Works for current AND future elements
```

We attach one listener to `document` that catches all clicks, checks if the target has `data-action="delete"`, and handles it. New elements automatically work.

### Tracing: Manual Logs vs Instrumentation

You may notice web handlers use `tracing::debug!()` calls instead of `#[tracing::instrument]` attributes used in REST API handlers. This is an **intentional design choice**:

**REST API handlers** (Chapter 8) use `#[tracing::instrument]`:
```rust
#[tracing::instrument(skip(state, dto))]
#[post("/api/v1/memos")]
pub async fn create_memo(
    state: web::Data<AppState>,
    dto: web::Json<CreateMemoDto>,
) -> Result<HttpResponse, AppError> {
    // Automatic span created with function name and parameters
}
```

**Web handlers** (this chapter) use manual `tracing::debug!()`:
```rust
#[post("/web/memos")]
pub async fn create_memo_web(
    state: web::Data<AppState>,
    form: web::Form<WebCreateMemoForm>,
) -> Result<HttpResponse, AppError> {
    tracing::debug!("Creating memo from web form");
    // Manual logging at specific points
}
```

**Why the difference?**

1. **Request logging already handled**: `TracingLogger` middleware (Chapter 1) automatically creates spans for all HTTP requests with method, path, and status code. This provides request-level tracing for both REST and web handlers.

2. **Different debugging needs**:
   - **REST API**: Function parameters are JSON data - often useful to log automatically
   - **Web handlers**: Form data contains user input that may be sensitive; manual logging gives control over what's logged

3. **Simplicity for HTML responses**: Web handlers primarily render templates. The important events (template render success/failure) are logged manually where they occur.

4. **Consistency with middleware approach**: Since `TracingLogger` already provides HTTP-level spans, adding `#[tracing::instrument]` would create nested spans that duplicate information.

**What you get from TracingLogger** (automatic):
```
INFO request{method=POST path="/web/memos" ...}
```

**What manual `tracing::debug!()` adds** (selective):
```
DEBUG Creating memo from web form
DEBUG memo_id=123e4567 Rendering edit memo form
```

**When to use each**:
- **Use `#[tracing::instrument]`**: Service layer, repository layer, complex business logic where you want automatic parameter logging
- **Use manual `tracing::debug!()`**: Handlers (covered by middleware), specific events you want to highlight

This approach balances comprehensive tracing with control over sensitive data logging.

## Step-by-Step Instructions

### Step 1: Create Web Form Structs

Web forms send string data that needs parsing and validation. Create form structs distinct from DTOs.

Add to `src/handlers/web.rs`:

```rust
use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use askama::Template;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::{CreateMemoDto, MemoResponseDto, PaginationParams, UpdateMemoDto},
    error::AppError,
    services::MemoService,
    state::AppState,
};

// Template structs from Chapter 10
#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    pub memos: Vec<MemoResponseDto>,
}

#[derive(Template)]
#[template(path = "components/memo_list.html")]
pub struct MemoListTemplate {
    pub memos: Vec<MemoResponseDto>,
}

#[derive(Template)]
#[template(path = "components/memo_item.html")]
pub struct MemoItemTemplate {
    pub memo: MemoResponseDto,
}

#[derive(Template)]
#[template(path = "components/memo_form.html")]
pub struct MemoFormTemplate {
    pub memo: Option<MemoResponseDto>,
}

/// Form data for creating a new memo (HTML form submission)
#[derive(Debug, Deserialize, Validate)]
pub struct WebCreateMemoForm {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    pub description: Option<String>,

    /// HTML datetime-local format: YYYY-MM-DDTHH:MM
    pub date_to: String,
}

/// Form data for updating a memo (HTML form submission)
#[derive(Debug, Deserialize, Validate)]
pub struct WebUpdateMemoForm {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    pub description: Option<String>,

    /// HTML datetime-local format: YYYY-MM-DDTHH:MM
    pub date_to: String,

    /// Checkbox sends value only if checked, None if unchecked
    pub completed: Option<String>,
}
```

**Why separate form structs?**

1. **Different validation**: HTML forms send strings, DTOs use DateTime<Utc>
2. **Checkbox handling**: HTML sends `Some("on")` if checked, `None` if unchecked
3. **Clear separation**: Form parsing vs business logic

### Step 2: Implement Homepage Handler

The homepage displays all memos in a full HTML page.

Add to `src/handlers/web.rs`:

```rust
/// Render homepage with list of memos
///
/// Returns full HTML page (extends base.html)
/// This is the initial page load - subsequent updates use AJAX
///
/// Note: We use manual tracing::debug!() instead of #[tracing::instrument]
/// because TracingLogger middleware already logs all HTTP requests
#[get("/")]
pub async fn index(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    tracing::debug!("Rendering index page");

    let service = MemoService::new(state.db.clone());
    let params = PaginationParams::default();

    // Fetch all memos using service layer
    let result = service.get_all_memos(params).await?;

    // Create template with data
    let template = IndexTemplate { memos: result.data };

    // Render template to HTML string
    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render index template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}
```

**Handler pattern**:
1. **Extract dependencies**: `state: web::Data<AppState>` for database access
2. **Log operation**: `tracing::debug!("...")` for manual logging
3. **Create service**: `MemoService::new(state.db.clone())`
4. **Fetch data**: `service.get_all_memos(params).await?`
5. **Create template struct**: `IndexTemplate { memos: result.data }`
6. **Render**: `template.render()` returns `Result<String, _>`
7. **Return response**: `HttpResponse::Ok().content_type("text/html").body(html)`

**Error handling**: Template render errors logged and converted to AppError.

**Tracing approach**: No `#[tracing::instrument]` attribute needed - `TracingLogger` middleware already creates spans for all HTTP requests (see "Tracing: Manual Logs vs Instrumentation" section above).

### Step 3: Implement Memo List Handler (AJAX)

This handler returns just the memo list HTML for AJAX updates (filters, sorting).

Add to `src/handlers/web.rs`:

```rust
/// Get filtered/sorted memo list as HTML fragment
///
/// Used by JavaScript for dynamic updates without page reload
/// Returns just the memo list component (not full page)
#[get("/web/memos")]
pub async fn get_memos_list(
    state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, AppError> {
    tracing::debug!("Fetching memos list for web");

    let service = MemoService::new(state.db.clone());

    // Use query parameters from URL (?completed=true&sort_by=title&order=asc)
    let result = service.get_all_memos(query.into_inner()).await?;

    // Render just the list component (no base layout)
    let template = MemoListTemplate { memos: result.data };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo list template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}
```

**Difference from index handler**:
- **Accepts query parameters**: `web::Query<PaginationParams>` for filtering/sorting
- **Returns fragment**: `MemoListTemplate` doesn't extend base.html
- **AJAX-friendly**: JavaScript can replace just the list portion

**JavaScript usage** (already in index.html from Chapter 10):
```javascript
fetch('/web/memos?completed=true&sort_by=title&order=asc')
    .then(res => res.text())
    .then(html => {
        document.getElementById('memo-list').innerHTML = html;
    });
```

### Step 4: Implement New Memo Form Handler

Returns the empty form HTML for creating a new memo.

Add to `src/handlers/web.rs`:

```rust
/// Get new memo form (empty form for creation)
///
/// Returns form component with memo = None
/// JavaScript loads this into a modal
#[get("/web/memos/new")]
pub async fn get_new_memo_form() -> Result<HttpResponse, AppError> {
    tracing::debug!("Rendering new memo form");

    // memo: None means create mode
    let template = MemoFormTemplate { memo: None };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo form template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}
```

**No database access**: This just renders an empty form. The template checks `{% match memo %}{% when None %}` and shows "New Memo" heading with empty fields.

### Step 5: Implement Create Memo Handler

Handles form submission to create a new memo.

Add to `src/handlers/web.rs`:

```rust
/// Create a new memo from web form submission
///
/// Accepts HTML form data, validates, creates memo, returns updated list
/// For AJAX: returns memo list HTML to replace existing list
#[post("/web/memos")]
pub async fn create_memo_web(
    state: web::Data<AppState>,
    form: web::Form<WebCreateMemoForm>,
) -> Result<HttpResponse, AppError> {
    tracing::debug!("Creating memo from web form");

    // Step 1: Validate form input
    form.validate()
        .map_err(|e| AppError::Validation(format!("Validation failed: {}", e)))?;

    // Step 2: Parse datetime string to DateTime<Utc>
    let date_to: DateTime<Utc> =
        chrono::NaiveDateTime::parse_from_str(&form.date_to, "%Y-%m-%dT%H:%M")
            .map_err(|_| {
                AppError::Validation("Invalid date format. Expected YYYY-MM-DDTHH:MM".to_string())
            })?
            .and_utc();

    // Step 3: Create service and DTO
    let service = MemoService::new(state.db.clone());

    let dto = CreateMemoDto {
        title: form.title.clone(),
        description: form.description.clone(),
        date_to,
    };

    // Step 4: Create memo via service
    service.create_memo(dto).await?;

    // Step 5: Return updated memo list (for AJAX)
    let params = PaginationParams::default();
    let result = service.get_all_memos(params).await?;

    let template = MemoListTemplate { memos: result.data };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo list template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}
```

**Form handling flow**:
1. **Extract**: `web::Form<WebCreateMemoForm>` auto-deserializes form data
2. **Validate**: `form.validate()` checks length constraints
3. **Parse dates**: Convert string to DateTime<Utc>
4. **Create DTO**: Map form to CreateMemoDto
5. **Call service**: `service.create_memo(dto)`
6. **Return updated list**: JavaScript will replace the list

**Why return the list, not the single item?**
- Ensures consistent ordering after creation
- Shows new memo in correct position (sorted by created_at)
- JavaScript can replace entire list with fresh data

### Step 6: Implement Edit Memo Form Handler

Returns the form pre-filled with existing memo data.

Add to `src/handlers/web.rs`:

```rust
/// Get edit memo form (pre-filled with existing memo data)
///
/// Returns form component with memo = Some(existing_memo)
/// JavaScript loads this into a modal
#[get("/web/memos/{id}/edit")]
pub async fn get_edit_memo_form(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    tracing::debug!(memo_id = %id, "Rendering edit memo form");

    // Fetch existing memo
    let service = MemoService::new(state.db.clone());
    let memo = service.get_memo_by_id(id).await?;

    // memo: Some(memo) means edit mode
    let template = MemoFormTemplate { memo: Some(memo) };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo form template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}
```

**Path parameter**: `web::Path<Uuid>` extracts `{id}` from URL `/web/memos/123e4567.../edit`

**Template logic**: The form template checks `{% match memo %}{% when Some with (m) %}` and:
- Shows "Edit Memo" heading
- Pre-fills fields: `value="{{ m.title }}"`
- Shows "Update" button instead of "Create"
- Displays checkbox for completion status

Same template, different mode!

### Step 7: Implement Update Memo Handler

Handles form submission to update an existing memo.

Add to `src/handlers/web.rs`:

```rust
/// Update a memo from web form submission
///
/// Accepts HTML form data, validates, updates memo, returns updated item
/// For AJAX: returns single memo HTML to replace existing item in list
#[put("/web/memos/{id}")]
pub async fn update_memo_web(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    form: web::Form<WebUpdateMemoForm>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    tracing::debug!(memo_id = %id, "Updating memo from web form");

    // Step 1: Validate form input
    form.validate()
        .map_err(|e| AppError::Validation(format!("Validation failed: {}", e)))?;

    // Step 2: Parse datetime
    let date_to: DateTime<Utc> =
        chrono::NaiveDateTime::parse_from_str(&form.date_to, "%Y-%m-%dT%H:%M")
            .map_err(|_| {
                AppError::Validation("Invalid date format. Expected YYYY-MM-DDTHH:MM".to_string())
            })?
            .and_utc();

    // Step 3: Convert checkbox to boolean
    // HTML checkboxes: checked = Some("on"), unchecked = None
    let completed = form.completed.is_some();

    // Step 4: Create service and DTO
    let service = MemoService::new(state.db.clone());

    let dto = UpdateMemoDto {
        title: form.title.clone(),
        description: form.description.clone(),
        date_to,
        completed,
    };

    // Step 5: Update via service
    let memo = service.update_memo(id, dto).await?;

    // Step 6: Return updated memo item (for AJAX)
    let template = MemoItemTemplate { memo };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo item template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}
```

**Key differences from create**:
- **Path parameter**: `{id}` to identify which memo
- **Checkbox parsing**: `form.completed.is_some()` converts Option<String> to bool
- **Returns single item**: JavaScript replaces just this memo's HTML

**Why return item, not list?**
- Efficient: Only update what changed
- Smooth UX: Memo stays in same position
- JavaScript: `document.getElementById('memo-{id}').outerHTML = html`

### Step 8: Implement Delete Memo Handler

Handles memo deletion.

Add to `src/handlers/web.rs`:

```rust
/// Delete a memo
///
/// For AJAX: returns empty body, JavaScript removes element from DOM
#[delete("/web/memos/{id}")]
pub async fn delete_memo_web(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    tracing::debug!(memo_id = %id, "Deleting memo from web");

    let service = MemoService::new(state.db.clone());
    service.delete_memo(id).await?;

    // Return empty success response
    // JavaScript will remove the element from DOM
    Ok(HttpResponse::Ok().body(""))
}
```

**Simple handler**: Just delete and return 200 OK with empty body.

**JavaScript handles UI update**:
```javascript
fetch(`/web/memos/${memoId}`, { method: 'DELETE' })
    .then(response => {
        if (response.ok) {
            document.getElementById(`memo-${memoId}`).remove();
        }
    });
```

No need to return HTML since element is removed entirely.

### Step 9: Implement Toggle Complete Handler

Toggles a memo's completion status.

Add to `src/handlers/web.rs`:

```rust
/// Toggle memo completion status
///
/// For AJAX: returns updated memo item HTML to replace existing
#[patch("/web/memos/{id}/toggle")]
pub async fn toggle_memo_complete_web(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    tracing::debug!(memo_id = %id, "Toggling memo completion status");

    let service = MemoService::new(state.db.clone());

    // Service method toggles completed: true ↔ false
    let memo = service.toggle_complete(id).await?;

    // Return updated memo item
    let template = MemoItemTemplate { memo };

    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to render memo item template");
            Err(AppError::Internal("Failed to render template".to_string()))
        }
    }
}
```

**PATCH method**: Semantic HTTP - PATCH for partial updates (just completion status).

**Returns updated item**: CSS classes change based on `completed` field, button text changes ("Complete" ↔ "Undo").

### Step 10: Register Web Handlers

Add web handlers to Actix Web app configuration.

Edit `src/main.rs`, update the App configuration. The exact middleware stack may vary depending on what you've already set up, but here's the essential configuration for web handlers:

```rust
HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(app_state.clone()))

        // Middleware (some may already be configured from earlier chapters)
        .wrap(TracingLogger::default())        // From Chapter 1
        // Note: Additional middleware (rate limiting, security headers, compression)
        // will be covered in Chapter 13 and may already exist in your codebase

        // Swagger UI (Chapter 9)
        .service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-docs/openapi.json", openapi.clone()),
        )

        // Web handlers (HTML pages and forms) - NEW IN THIS CHAPTER
        .service(handlers::index)                         // or handlers::web::index
        .service(handlers::get_memos_list)
        .service(handlers::get_new_memo_form)
        .service(handlers::create_memo_web)
        .service(handlers::get_edit_memo_form)
        .service(handlers::update_memo_web)
        .service(handlers::delete_memo_web)
        .service(handlers::toggle_memo_complete_web)

        // Health checks (Chapter 4)
        .service(handlers::health_check)                  // or handlers::health::health_check
        .service(handlers::ready)

        // REST API (Chapter 8)
        .service(handlers::list_memos)                    // or handlers::memos::list_memos
        .service(handlers::get_memo)
        .service(handlers::create_memo)
        .service(handlers::update_memo)
        .service(handlers::patch_memo)
        .service(handlers::delete_memo)
        .service(handlers::toggle_complete)
})
```

**Note on imports**: If you've set up re-exports in `handlers/mod.rs` (from Step 11), you can use `handlers::index`. Otherwise, use the full path `handlers::web::index`.

**Note on static files**: Static file serving (`.service(actix_files::Files::new("/static", "./static"))`) will be added in Chapter 12 when we create the actual CSS files. If it's already in your configuration, that's fine - the files will be created next chapter.

**Route organization**:
- `/` → Homepage (new in this chapter)
- `/web/memos/*` → Web UI endpoints (new in this chapter)
- `/api/v1/memos/*` → REST API endpoints (Chapter 8)
- `/health`, `/ready` → Monitoring (Chapter 4)
- `/swagger-ui/` → API docs (Chapter 9)

### Step 11: Verify Handler Registration

Ensure handlers module exports web handlers.

Edit `src/handlers/mod.rs`:

```rust
pub mod health;
pub mod memos;
pub mod web;

// Re-export for convenience in main.rs
pub use health::{health_check, ready};
pub use memos::{
    create_memo, delete_memo, get_memo, list_memos, patch_memo, toggle_complete, update_memo,
};
pub use web::{
    create_memo_web, delete_memo_web, get_edit_memo_form, get_memos_list, get_new_memo_form,
    index, toggle_memo_complete_web, update_memo_web,
};
```

This allows importing handlers directly: `handlers::index` instead of `handlers::web::index`.

### Step 12: Test Web Handlers

Build and run the application:

```bash
# Compile
cargo build

# Run (ensure PostgreSQL is running)
cargo run
```

**Test in browser**:

1. **Navigate to homepage**: http://localhost:3737/
   - Should see beautifully styled memo list with CSS from Chapter 11
   - Page should have styled header, footer, filters
   - Colors, spacing, and layout should match the CSS you created

2. **Click "New Memo"**:
   - Styled modal should appear with empty form
   - (requires JavaScript)
   - Modal overlay, rounded corners, and form styling visible

3. **Create a memo** (fill form and submit):
   - List should update with new styled memo card
   - Smooth transition, no page reload
   - Button hover effects and colors visible

4. **Test without JavaScript** (disable in browser dev tools):
   - New memo button won't work (needs JS)
   - Direct navigation to `/web/memos/new` shows form
   - Form submit causes page reload (baseline experience)

5. **Test filters**:
   - Change "All Memos" to "Incomplete"
   - List updates dynamically with styled cards

6. **Test toggle complete**:
   - Click "Complete" button on memo
   - Memo visual changes (opacity reduces, badge changes color)
   - Button text changes to "Undo"
   - Smooth CSS transitions visible

7. **Test edit**:
   - Click "Edit" on a memo
   - Styled form opens with pre-filled data
   - Submit updates memo in-place with styling

8. **Test delete**:
   - Click "Delete" on a memo
   - Confirmation dialog appears
   - Memo disappears from list

**Important**: With the CSS from Chapter 11 now connected to functional handlers, you have a complete, professional-looking web application!

## Checkpoint

At this point, you should have:

**Implemented handlers**:
- `GET /` - Homepage (full page)
- `GET /web/memos` - Memo list fragment (AJAX)
- `GET /web/memos/new` - New memo form
- `POST /web/memos` - Create memo
- `GET /web/memos/{id}/edit` - Edit memo form
- `PUT /web/memos/{id}` - Update memo
- `DELETE /web/memos/{id}` - Delete memo
- `PATCH /web/memos/{id}/toggle` - Toggle complete

**Handler patterns**:
- Template rendering with error handling
- Form data extraction and validation
- DateTime parsing from HTML format
- Checkbox conversion to boolean
- Full pages vs partial fragments

**Verification**:
```bash
# App runs without errors
cargo run

# Visit homepage
curl http://localhost:3737/
# Should return HTML with <!DOCTYPE html>

# Get memo list fragment
curl http://localhost:3737/web/memos
# Should return HTML without <!DOCTYPE> (just list component)
```

**What works**:
- Server-side rendered pages
- Form submissions (with JavaScript)
- AJAX partial updates
- Progressive enhancement (works without JS for basic operations)

**What doesn't work yet** (if CSS not added):
- Styling (covered in Chapter 12)
- Visual polish
- Modal animations

## Common Issues and Solutions

### Issue: Template render fails with "template not found"

**Symptom**:
```
Failed to render index template
```

**Cause**: Template files missing or Askama not finding them.

**Solution**:
```bash
# Verify templates exist
ls templates/pages/index.html
ls templates/components/memo_list.html

# Rebuild to recompile templates
cargo clean
cargo build
```

Askama compiles templates at build time. If templates were added after build, rebuild is needed.

### Issue: Form submission returns validation error

**Symptom**:
```
Validation failed: title: length
```

**Cause**: Form data doesn't meet validation constraints.

**Solution**:
Check form constraints match DTO/form struct:
```rust
#[validate(length(min = 1, max = 200))]
pub title: String,
```

HTML form should have:
```html
<input name="title" maxlength="200" required>
```

### Issue: DateTime parsing fails

**Symptom**:
```
Invalid date format. Expected YYYY-MM-DDTHH:MM
```

**Cause**: HTML form sends datetime in wrong format.

**Solution**:
Ensure input type is `datetime-local`:
```html
<input type="datetime-local" name="date_to" required>
```

This sends format: `2025-01-15T10:30` which matches our parser.

**Debug**:
```rust
tracing::debug!(date_str = %form.date_to, "Parsing datetime");
```

### Issue: Checkbox always false

**Symptom**: Memo always created as incomplete, even when checkbox checked.

**Cause**: Not handling checkbox Option<String> correctly.

**Solution**:
```rust
// HTML checkbox: checked = Some("on"), unchecked = None
let completed = form.completed.is_some();
```

Don't check the string value, just presence:
```rust
// ✗ Wrong
let completed = form.completed == Some("true");

// ✓ Correct
let completed = form.completed.is_some();
```

### Issue: AJAX updates don't work

**Symptom**: Clicking buttons does nothing, or causes page reload.

**Cause**: JavaScript errors or event handlers not attached.

**Solution**:
1. **Open browser console** (F12) and check for errors
2. **Verify JavaScript loaded**: View page source, check `<script>` block exists
3. **Test event delegation**:
   ```javascript
   document.addEventListener('click', function(e) {
       console.log('Click:', e.target);
   });
   ```

### Issue: Modal doesn't appear

**Symptom**: "New Memo" button does nothing.

**Cause**: JavaScript function not defined or CSS modal not styled.

**Solution**:
1. Check `showModal` function exists in index.html
2. Verify modal element exists: `<div id="memo-form-modal">`
3. Check CSS (Chapter 12): `.modal { display: none; }`

Without CSS, modal might be invisible or always visible.

### Issue: Updated memo doesn't reflect changes

**Symptom**: Edit form submits but changes don't appear.

**Cause**: JavaScript not replacing HTML correctly.

**Solution**:
Check element ID matches:
```html
<div id="memo-{{ memo.id }}">...</div>
```

JavaScript should use:
```javascript
document.getElementById(`memo-${memoId}`).outerHTML = html;
```

Use `outerHTML` (replaces entire element including div) not `innerHTML` (replaces only contents).

## Code Review

Let's review the complete web handler implementation.

### Principles Demonstrated

**Separation of Concerns**
- **Handlers**: Parse HTTP requests, validate input, render templates
- **Services**: Business logic, transactions, data transformations
- **Templates**: Presentation logic, HTML structure
- **JavaScript**: Client-side interactivity (optional enhancement)

**Progressive Enhancement**
- Base functionality works without JavaScript
- JavaScript adds smooth UX (no page reloads)
- Graceful degradation if JS fails

**Type Safety**
- Form structs with validation
- Compile-time template checking
- Strong typing throughout

**Consistency**
- All handlers follow same pattern
- Error handling centralized in AppError
- Logging with tracing at debug level

**DRY (Don't Repeat Yourself)**
- Single template for create/edit forms (memo: Option)
- Reusable components (memo_item, memo_list)
- Shared service layer with REST API

### Architecture Review

**Request flow** (AJAX create memo):
```
1. User submits form
2. JavaScript: fetch('/web/memos', { method: 'POST', body: formData })
3. Actix: create_memo_web handler
4. Extract: web::Form<WebCreateMemoForm>
5. Validate: form.validate()
6. Parse: DateTime from string
7. Service: create_memo(dto)
8. Render: MemoListTemplate with updated memos
9. Return: HTML string
10. JavaScript: update DOM with new HTML
11. User sees: Updated list without page reload
```

**Data transformations**:
```
HTML Form Data (application/x-www-form-urlencoded)
    ↓ Actix web::Form deserializer
WebCreateMemoForm (validated)
    ↓ Handler parsing
CreateMemoDto (business object)
    ↓ Service layer
ActiveModel (database object)
    ↓ Repository
Database INSERT
    ↓ Repository
Entity (database result)
    ↓ Service
MemoResponseDto (API response)
    ↓ Template
HTML String
```

### Structure Review

**Handler responsibilities**:
1. **Extract data**: path params, query params, form data, state
2. **Validate**: input validation with validator crate
3. **Parse**: format conversions (strings to DateTime, etc.)
4. **Call service**: delegate business logic
5. **Render template**: create template struct and render
6. **Handle errors**: convert to AppError, log with tracing
7. **Return response**: HttpResponse with appropriate status

**What handlers don't do**:
- Database access (that's repository layer)
- Business logic (that's service layer)
- Complex validation (beyond input format validation)
- HTML generation (that's templates)

**Form handling pattern**:
```rust
pub async fn handler(
    state: web::Data<AppState>,         // Dependency injection
    form: web::Form<FormStruct>,        // Automatic deserialization
) -> Result<HttpResponse, AppError> {   // Consistent error handling

    // 1. Validate
    form.validate()?;

    // 2. Parse/transform
    let data = parse_form_data(&form)?;

    // 3. Business logic via service
    let service = MemoService::new(state.db.clone());
    let result = service.do_something(data).await?;

    // 4. Render response
    let template = SomeTemplate { data: result };
    Ok(HttpResponse::Ok().body(template.render()?))
}
```

Every handler follows this pattern.

## Testing

### Integration Tests

Create `tests/web_tests.rs`:

```rust
mod common;

use actix_web::{test, web, App};
use actix_web_template::{
    handlers::web::{
        create_memo_web, delete_memo_web, get_edit_memo_form, get_memos_list, get_new_memo_form,
        index, toggle_memo_complete_web, update_memo_web,
    },
    services::MemoService,
};
use chrono::Utc;
use common::{fixtures::create_test_memo_dto, setup_test_state};

#[tokio::test]
async fn test_index_page() {
    let state = setup_test_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(index)
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's a complete HTML page
    assert!(html.contains("<!DOCTYPE html") || html.contains("<html"));
}

#[tokio::test]
async fn test_get_memos_list() {
    let state = setup_test_state().await;

    // Create test memo
    let service = MemoService::new(state.db.clone());
    let dto = create_test_memo_dto("Web List Test", None);
    let created = service.create_memo(dto).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(get_memos_list),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/web/memos?limit=10&offset=0")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Verify memo appears in list
    assert!(html.contains("Web List Test"));

    // Cleanup
    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_get_new_memo_form() {
    let app = test::init_service(
        App::new().service(get_new_memo_form)
    ).await;

    let req = test::TestRequest::get()
        .uri("/web/memos/new")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's a form
    assert!(html.contains("<form") || html.contains("form"));
    assert!(html.contains("New Memo"));
}

#[tokio::test]
async fn test_create_memo_web() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(create_memo_web),
    )
    .await;

    let date_str = Utc::now().format("%Y-%m-%dT%H:%M").to_string();

    let req = test::TestRequest::post()
        .uri("/web/memos")
        .set_form([
            ("title", "Web Created Memo"),
            ("description", "Test description"),
            ("date_to", &date_str),
        ])
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify memo was created
    let params = actix_web_template::dto::PaginationParams::default();
    let result = service.get_all_memos(params).await.unwrap();

    assert!(result.data.iter().any(|m| m.title == "Web Created Memo"));

    // Cleanup
    let created = result.data.iter().find(|m| m.title == "Web Created Memo").unwrap();
    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_update_memo_web() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    // Create test memo
    let dto = create_test_memo_dto("Update Test", None);
    let created = service.create_memo(dto).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(update_memo_web),
    )
    .await;

    let date_str = Utc::now().format("%Y-%m-%dT%H:%M").to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/web/memos/{}", created.id))
        .set_form([
            ("title", "Updated Title"),
            ("description", "Updated description"),
            ("date_to", &date_str),
            ("completed", "on"),  // Checkbox checked
        ])
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify update
    let updated = service.get_memo_by_id(created.id).await.unwrap();
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.completed, true);

    // Cleanup
    service.delete_memo(created.id).await.ok();
}

#[tokio::test]
async fn test_delete_memo_web() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    // Create test memo
    let dto = create_test_memo_dto("Delete Test", None);
    let created = service.create_memo(dto).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(delete_memo_web),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/web/memos/{}", created.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify deleted
    let result = service.get_memo_by_id(created.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_toggle_memo_complete() {
    let state = setup_test_state().await;
    let service = MemoService::new(state.db.clone());

    // Create incomplete memo
    let dto = create_test_memo_dto("Toggle Test", None);
    let created = service.create_memo(dto).await.unwrap();
    assert_eq!(created.completed, false);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(toggle_memo_complete_web),
    )
    .await;

    // Toggle to completed
    let req = test::TestRequest::patch()
        .uri(&format!("/web/memos/{}/toggle", created.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let updated = service.get_memo_by_id(created.id).await.unwrap();
    assert_eq!(updated.completed, true);

    // Cleanup
    service.delete_memo(created.id).await.ok();
}
```

Run tests:

```bash
cargo test web_tests
```

**Expected output**:
```
running 7 tests
test test_index_page ... ok
test test_get_memos_list ... ok
test test_get_new_memo_form ... ok
test test_create_memo_web ... ok
test test_update_memo_web ... ok
test test_delete_memo_web ... ok
test test_toggle_memo_complete ... ok

test result: ok. 7 passed; 0 failed
```

## Summary

You've successfully implemented a complete web interface with server-side rendering:

**Key achievements**:
1. **Eight web handlers**: Full CRUD operations via HTML forms
2. **Progressive enhancement**: Works without JavaScript, enhanced with it
3. **Form handling**: Validation, parsing, checkbox handling, datetime conversion
4. **Template rendering**: Full pages and partial fragments
5. **AJAX support**: Dynamic updates without page reloads
6. **Error handling**: Consistent AppError usage with logging
7. **Integration tests**: Comprehensive test coverage for all handlers

**Patterns learned**:
- **Handler structure**: Extract → Validate → Parse → Service → Render → Respond
- **Form to DTO conversion**: Web forms → Form structs → DTOs → Service
- **Full vs partial renders**: Complete pages (extends base) vs fragments (components)
- **Redirect After Post**: For traditional forms (not used here due to AJAX)
- **Event delegation**: JavaScript pattern for dynamic content

**How this fits into the application**:
- **Handlers** (Chapter 11) connect **Templates** (Chapter 10) to **Services** (Chapter 7)
- **Services** use **Repositories** (Chapter 6) for database access
- **DTOs** (Chapter 5) transfer data between layers
- **Middleware** (Chapter 3) wraps all handlers (logging, security, rate limiting)
- **Error handling** (Chapter 3) catches and converts errors consistently

Web UI is complete. The app is functional but needs styling (Chapter 12).

## Next Steps

In **Chapter 13: Security Enhancements**, you'll:
- Implement rate limiting with actix-governor
- Add HTML sanitization with ammonia
- Configure Content Security Policy headers
- Set up HSTS for production
- Test security features
- Audit for common vulnerabilities

The app is functional and beautiful. Now it's time to harden it against security threats.

## Additional Resources

### Official Documentation
- [Actix Web Extractors](https://actix.rs/docs/extractors/) - Form, Query, Path, Json
- [Actix Web Response](https://actix.rs/docs/response/) - HttpResponse builders
- [Askama in Actix](https://djc.github.io/askama/integrations.html#actix-web) - Template integration
- [Serde Deserialize](https://serde.rs/derive.html) - Form struct deserialization

### Form Handling
- [HTML Forms](https://developer.mozilla.org/en-US/docs/Learn/Forms) - MDN comprehensive guide
- [Form Validation](https://developer.mozilla.org/en-US/docs/Learn/Forms/Form_validation) - Client and server-side
- [HTML Input Types](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input) - datetime-local, checkbox, etc.

### JavaScript and AJAX
- [Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch) - Modern HTTP requests
- [FormData](https://developer.mozilla.org/en-US/docs/Web/API/FormData) - Form serialization
- [Event Delegation](https://javascript.info/event-delegation) - Pattern for dynamic content

### Progressive Enhancement
- [Progressive Enhancement](https://developer.mozilla.org/en-US/docs/Glossary/Progressive_Enhancement) - MDN definition
- [Resilient Web Design](https://resilientwebdesign.com/) - Philosophy and patterns
- [The Web Without JS](https://www.kryogenix.org/code/browser/everyonehasjs.html) - Why progressive enhancement matters

### Related Topics
- [RESTful APIs vs HTML Forms](https://htmx.org/essays/how-did-rest-come-to-mean-the-opposite-of-rest/) - Understanding trade-offs
- [Server-Side Rendering Benefits](https://web.dev/rendering-on-the-web/) - Performance and SEO
- [HTMX](https://htmx.org/) - Alternative approach to AJAX (extends HTML)
