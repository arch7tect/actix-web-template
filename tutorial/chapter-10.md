# Chapter 10: Askama Templates - Server-Side Rendering

## Overview

Askama is a type-safe, compile-time template engine for Rust that brings the power of Jinja2-style templating with compile-time verification. Unlike runtime template engines, Askama compiles your templates directly into your Rust binary, catching errors at build time and delivering exceptional runtime performance with zero overhead.

In this chapter, you'll build a complete set of HTML templates for the memo application: a base layout with header and footer, reusable components for memos, and complete pages for the user interface. You'll learn template inheritance, component composition, control flow, and how Askama's compile-time checking prevents common template errors before your code ever runs.

By the end of this chapter, you'll have a fully functional HTML template system that renders server-side, escapes user content automatically for XSS protection, and integrates seamlessly with Rust's type system.

## Prerequisites

### Completed Chapters
- Chapter 0: Prerequisites and Environment Setup
- Chapter 1: Core Application Setup
- Chapter 5: DTOs and Validation (for MemoResponseDto)

### Required Knowledge
- Basic HTML and CSS
- Understanding of template engines (e.g., Jinja2, Handlebars)
- Familiarity with Rust structs and derive macros

### System Requirements
- Rust 1.70+ (for Askama 0.14)
- Text editor with HTML support

## Learning Objectives

By the end of this chapter, you will be able to:

1. Configure Askama for Actix Web applications
2. Create base layouts with template inheritance
3. Build reusable template components
4. Use Askama's control flow features (if, for, match)
5. Handle Option types in templates with pattern matching
6. Create type-safe template structs in Rust
7. Leverage compile-time template checking for error prevention
8. Understand automatic XSS protection through HTML escaping

## Concepts Covered

### Template Engines and Server-Side Rendering

**Template engines** separate presentation (HTML) from logic (Rust code), allowing designers and developers to work independently. Unlike client-side frameworks (React, Vue), server-side rendering (SSR) generates complete HTML on the server and sends it to the browser, offering:

- **Better SEO**: Search engines see complete content immediately
- **Faster initial load**: No waiting for JavaScript to render
- **Progressive enhancement**: Works without JavaScript, enhanced with it
- **Reduced client load**: Less processing on user devices

### Compile-Time vs Runtime Templates

Most template engines (Handlebars, Tera) parse templates at **runtime**:
- Load template files when the server starts
- Parse template syntax for each render
- No type checking until runtime
- Runtime errors possible

Askama uses **compile-time** templates:
- Templates parsed during `cargo build`
- Compiled directly into Rust binary
- Full type checking at compile time
- No template files loaded at runtime
- Zero runtime overhead

```rust
// Compile-time: Template struct must match template file
#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    pub memos: Vec<MemoResponseDto>,  // Type must match template usage
}

// Compiler catches mismatches:
// - Missing fields in template
// - Wrong types in expressions
// - Invalid syntax in template
```

### Template Inheritance and Composition

**Template inheritance** creates a hierarchy where child templates extend parent templates:

```
base.html (layout)
├── pages/index.html (homepage)
├── pages/error.html (error page)
└── ... (other pages)
```

**Base template** defines structure with blocks:
```html
<html>
  <head>{% block title %}Default Title{% endblock %}</head>
  <body>
    {% include "partials/header.html" %}
    <main>{% block content %}{% endblock %}</main>
    {% include "partials/footer.html" %}
  </body>
</html>
```

**Child template** extends base and fills blocks:
```html
{% extends "base.html" %}
{% block title %}My Page{% endblock %}
{% block content %}<h1>Hello</h1>{% endblock %}
```

**Result** combines both:
```html
<html>
  <head>My Page</head>
  <body>
    [header content]
    <main><h1>Hello</h1></main>
    [footer content]
  </body>
</html>
```

### Askama Template Syntax

Askama uses Jinja2-like syntax with three delimiters:

**Variable interpolation** - Outputs values (automatically HTML-escaped):
```html
{{ variable }}
{{ user.name }}
{{ memo.title }}
```

**Expressions** - Conditionals, loops, etc.:
```html
{% if condition %}...{% endif %}
{% for item in items %}...{% endfor %}
{% match option %}...{% endmatch %}
```

**Comments** - Not rendered in output:
```html
{# This is a comment #}
```

### Control Flow in Templates

**Conditionals**:
```html
{% if memo.completed %}
    <span class="status-completed">Completed</span>
{% else %}
    <span class="status-pending">Pending</span>
{% endif %}
```

**Loops**:
```html
{% for memo in memos %}
    <div>{{ memo.title }}</div>
{% endfor %}
```

**Pattern Matching** (for Option and Result):
```html
{% match memo.description %}
    {% when Some with (desc) %}
        <p>{{ desc }}</p>
    {% when None %}
        <p>No description</p>
{% endmatch %}
```

### Automatic XSS Protection

Askama automatically HTML-escapes all variables to prevent Cross-Site Scripting (XSS):

```html
<!-- User input: <script>alert('xss')</script> -->
{{ memo.title }}
<!-- Rendered: &lt;script&gt;alert('xss')&lt;/script&gt; -->
```

This happens automatically for all `{{ }}` interpolations. To output raw HTML (dangerous!), use the `|safe` filter:
```html
{{ trusted_html|safe }}  {# Only for sanitized content! #}
```

Our application sanitizes user input with the `ammonia` crate before storing, providing defense-in-depth.

### Template Organization Patterns

**Directory structure**:
```
templates/
├── base.html              # Layout with common structure
├── pages/                 # Full page templates
│   ├── index.html         # Homepage
│   └── error.html         # Error page
├── components/            # Reusable UI components
│   ├── memo_list.html     # List of memos
│   ├── memo_item.html     # Single memo display
│   └── memo_form.html     # Create/edit form
└── partials/              # Small reusable pieces
    ├── header.html        # Site header
    └── footer.html        # Site footer
```

**Usage patterns**:
- **extends**: Create page templates extending base layout
- **include**: Insert partials (header, footer) and components
- **blocks**: Override specific sections in child templates

## Step-by-Step Instructions

### Step 1: Add Askama Dependency

Update `Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
askama = "0.14"
```

Askama provides:
- Template derive macro for creating template structs
- Compile-time template parsing and validation
- HTML escaping for XSS protection
- Integration with actix-web via `impl Responder`

Build to download the dependency:

```bash
cargo build
```

### Step 2: Create Templates Directory Structure

Create the directory hierarchy:

```bash
mkdir -p templates/pages
mkdir -p templates/components
mkdir -p templates/partials
```

This organization separates:
- **pages/**: Full-page templates that extend base layout
- **components/**: Reusable UI components (memos, forms)
- **partials/**: Small shared pieces (header, footer)

### Step 3: Create Base Layout Template

Create `templates/base.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Memos App{% endblock %}</title>
    <link rel="stylesheet" href="/static/css/style.css">
    {% block head_scripts %}{% endblock %}
</head>
<body>
    {% include "partials/header.html" %}

    <main>
        {% block content %}{% endblock %}
    </main>

    {% include "partials/footer.html" %}
</body>
</html>
```

**Key features**:
- **{% block title %}**: Child templates can override the title
- **{% block head_scripts %}**: Injection point for page-specific JavaScript
- **{% include "partials/header.html" %}**: Inserts header component
- **{% block content %}**: Main content area for child templates
- **Static CSS link**: References `/static/css/style.css` (Chapter 12)

This template provides a consistent layout for all pages: every page gets the header, footer, and style sheet automatically.

### Step 4: Create Header Partial

Create `templates/partials/header.html`:

```html
<header>
    <div class="container">
        <h1><a href="/">Memos App</a></h1>
        <nav>
            <ul>
                <li><a href="/">Home</a></li>
                <li><a href="/swagger-ui/">API Docs</a></li>
            </ul>
        </nav>
    </div>
</header>
```

This partial provides:
- Site branding with home link
- Navigation to key sections
- Links to API documentation (from Chapter 9)

Partials are included with `{% include %}` and render inline where referenced.

### Step 5: Create Footer Partial

Create `templates/partials/footer.html`:

```html
<footer>
    <div class="container">
        <p>&copy; 2025 Memos App. Built with Actix Web.</p>
    </div>
</footer>
```

Simple footer with copyright and attribution. Every page includes this via the base template.

### Step 6: Create Homepage Template

Create `templates/pages/index.html`:

```html
{% extends "base.html" %}

{% block title %}Memos - Home{% endblock %}

{% block head_scripts %}
<script>
function showModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.style.display = 'block';
    }
}

function closeModal(modalId) {
    document.getElementById(modalId).style.display = 'none';
}

function loadNewMemoForm() {
    fetch('/web/memos/new')
        .then(response => response.text())
        .then(html => {
            document.getElementById('memo-form-container').innerHTML = html;
            showModal('memo-form-modal');
        });
}

document.addEventListener('DOMContentLoaded', function() {
    const modal = document.getElementById('memo-form-modal');
    if (modal) {
        const closeBtn = modal.querySelector('.close');
        if (closeBtn) {
            closeBtn.onclick = function() {
                modal.style.display = 'none';
            };
        }

        window.onclick = function(event) {
            if (event.target === modal) {
                modal.style.display = 'none';
            }
        };
    }

    function updateMemoList() {
        const completed = document.getElementById('filter-completed').value;
        const sortBy = document.getElementById('sort-by').value;
        const order = document.getElementById('order').value;

        const params = new URLSearchParams();
        if (completed) params.append('completed', completed);
        if (sortBy) params.append('sort_by', sortBy);
        if (order) params.append('order', order);

        const url = '/web/memos?' + params.toString();

        fetch(url)
            .then(response => response.text())
            .then(html => {
                document.getElementById('memo-list').innerHTML = html;
            });
    }

    document.getElementById('filter-completed').addEventListener('change', updateMemoList);
    document.getElementById('sort-by').addEventListener('change', updateMemoList);
    document.getElementById('order').addEventListener('change', updateMemoList);

    document.addEventListener('click', function(e) {
        const target = e.target;
        if (!target.dataset.action) return;

        const action = target.dataset.action;
        const memoId = target.dataset.memoId;

        if (action === 'edit') {
            fetch(`/web/memos/${memoId}/edit`)
                .then(response => response.text())
                .then(html => {
                    document.getElementById('memo-form-container').innerHTML = html;
                    showModal('memo-form-modal');
                });
        } else if (action === 'toggle') {
            fetch(`/web/memos/${memoId}/toggle`, { method: 'PATCH' })
                .then(response => response.text())
                .then(html => {
                    document.getElementById(`memo-${memoId}`).outerHTML = html;
                });
        } else if (action === 'delete') {
            if (confirm('Are you sure you want to delete this memo?')) {
                fetch(`/web/memos/${memoId}`, { method: 'DELETE' })
                    .then(response => {
                        if (response.ok) {
                            document.getElementById(`memo-${memoId}`).remove();
                        }
                    });
            }
        }
    });

    document.addEventListener('submit', function(e) {
        if (e.target.id === 'memo-form') {
            e.preventDefault();
            const form = e.target;
            const memoId = form.dataset.memoId;
            const formData = new FormData(form);

            const url = memoId ? `/web/memos/${memoId}` : '/web/memos';
            const method = memoId ? 'PUT' : 'POST';

            const params = new URLSearchParams();
            for (const [key, value] of formData.entries()) {
                params.append(key, value);
            }

            fetch(url, {
                method: method,
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                },
                body: params.toString()
            })
            .then(response => response.text())
            .then(html => {
                if (memoId) {
                    document.getElementById(`memo-${memoId}`).outerHTML = html;
                } else {
                    document.getElementById('memo-list').innerHTML = html;
                }
                closeModal('memo-form-modal');
            });
        }
    });
});
</script>
{% endblock %}

{% block content %}
<div class="container">
    <div class="page-header">
        <h2>My Memos</h2>
        <button
            id="new-memo-btn"
            class="btn btn-primary"
            onclick="loadNewMemoForm()">
            New Memo
        </button>
    </div>

    <div class="filters">
        <form id="filter-form">
            <select
                id="filter-completed"
                name="completed">
                <option value="" selected>All Memos</option>
                <option value="false">Incomplete</option>
                <option value="true">Completed</option>
            </select>

            <select
                id="sort-by"
                name="sort_by">
                <option value="created_at" selected>Created At</option>
                <option value="date_to">Due Date</option>
                <option value="title">Title</option>
            </select>

            <select
                id="order"
                name="order">
                <option value="desc" selected>Descending</option>
                <option value="asc">Ascending</option>
            </select>
        </form>
    </div>

    <div id="memo-list">
        {% include "components/memo_list.html" %}
    </div>
</div>

<div id="memo-form-modal" class="modal" style="display: none;">
    <div class="modal-content">
        <span class="close">&times;</span>
        <div id="memo-form-container"></div>
    </div>
</div>
{% endblock %}
```

**Template structure**:
- **{% extends "base.html" %}**: Inherits layout structure
- **{% block title %}**: Overrides default title
- **{% block head_scripts %}**: Adds page-specific JavaScript for:
  - Modal handling (create/edit forms)
  - AJAX operations (create, update, delete, toggle)
  - Filter/sort controls with live updates
  - Event delegation for dynamic content
- **{% block content %}**: Main page content with:
  - Page header with "New Memo" button
  - Filter controls (completion, sort, order)
  - Memo list container (includes component)
  - Hidden modal for forms

**Progressive enhancement**: The page works with just HTML (server-rendered), enhanced with JavaScript for smooth UX (no page reloads).

### Step 7: Create Memo List Component

Create `templates/components/memo_list.html`:

```html
{% if memos.is_empty() %}
    <div class="empty-state">
        <p>No memos found. Create your first memo!</p>
    </div>
{% else %}
    {% for memo in memos %}
        {% include "components/memo_item.html" %}
    {% endfor %}
{% endif %}
```

**Control flow**:
- **{% if memos.is_empty() %}**: Conditional rendering based on collection state
- **{% for memo in memos %}**: Loop over memo collection
- **{% include "components/memo_item.html" %}**: Render each memo with item template

This component handles both empty and populated states gracefully.

### Step 8: Create Memo Item Component

Create `templates/components/memo_item.html`:

```html
<div class="memo-item {% if memo.completed %}completed{% endif %}" id="memo-{{ memo.id }}">
    <div class="memo-header">
        <h3 class="memo-title">{{ memo.title }}</h3>
        <div class="memo-actions">
            <button
                class="btn btn-sm btn-toggle"
                data-action="toggle"
                data-memo-id="{{ memo.id }}">
                {% if memo.completed %}Undo{% else %}Complete{% endif %}
            </button>
            <button
                class="btn btn-sm btn-edit"
                data-action="edit"
                data-memo-id="{{ memo.id }}">
                Edit
            </button>
            <button
                class="btn btn-sm btn-danger"
                data-action="delete"
                data-memo-id="{{ memo.id }}">
                Delete
            </button>
        </div>
    </div>
    {% match memo.description %}
    {% when Some with (desc) %}
    <p class="memo-description">{{ desc }}</p>
    {% when None %}
    {% endmatch %}
    <div class="memo-footer">
        <span class="memo-date">Due: {{ memo.date_to }}</span>
        <span class="memo-status {% if memo.completed %}status-completed{% else %}status-pending{% endif %}">
            {% if memo.completed %}Completed{% else %}Pending{% endif %}
        </span>
    </div>
</div>
```

**Askama features demonstrated**:

**Conditional CSS classes**:
```html
<div class="memo-item {% if memo.completed %}completed{% endif %}">
```

**Variable interpolation**:
```html
{{ memo.title }}
{{ memo.id }}
```

**Pattern matching for Option types**:
```html
{% match memo.description %}
    {% when Some with (desc) %}
        <p>{{ desc }}</p>
    {% when None %}
        {# Render nothing when None #}
{% endmatch %}
```

This is safer than unwrapping and handles Option elegantly.

**Data attributes for JavaScript**:
```html
data-action="toggle"
data-memo-id="{{ memo.id }}"
```

Enables event delegation in vanilla JavaScript without frameworks.

### Step 9: Create Memo Form Component

Create `templates/components/memo_form.html`:

```html
<form
    id="memo-form"
    data-memo-id="{% match memo %}{% when Some with (m) %}{{ m.id }}{% when None %}{% endmatch %}"
    class="memo-form">

    <h3>{% match memo %}{% when Some with (_) %}Edit Memo{% when None %}New Memo{% endmatch %}</h3>

    <div class="form-group">
        <label for="title">Title *</label>
        <input
            type="text"
            id="title"
            name="title"
            maxlength="200"
            required
            {% match memo %}{% when Some with (m) %}value="{{ m.title }}"{% when None %}{% endmatch %}
            placeholder="Enter memo title">
    </div>

    <div class="form-group">
        <label for="description">Description</label>
        <textarea
            id="description"
            name="description"
            maxlength="1000"
            rows="4"
            placeholder="Enter memo description (optional)">{% match memo %}{% when Some with (m) %}{% match m.description %}{% when Some with (desc) %}{{ desc }}{% when None %}{% endmatch %}{% when None %}{% endmatch %}</textarea>
    </div>

    <div class="form-group">
        <label for="date_to">Due Date *</label>
        <input
            type="datetime-local"
            id="date_to"
            name="date_to"
            required
            {% match memo %}{% when Some with (m) %}value="{{ m.date_to_local_format() }}"{% when None %}{% endmatch %}>
    </div>

    {% match memo %}
    {% when Some with (m) %}
    <div class="form-group">
        <label>
            <input
                type="checkbox"
                name="completed"
                {% if m.completed %}checked{% endif %}>
            Mark as completed
        </label>
    </div>
    {% when None %}
    {% endmatch %}

    <div class="form-actions">
        <button type="submit" class="btn btn-primary">
            {% match memo %}{% when Some with (_) %}Update{% when None %}Create{% endmatch %}
        </button>
        <button type="button" class="btn btn-secondary" onclick="closeModal('memo-form-modal')">
            Cancel
        </button>
    </div>
</form>
```

**Dual-purpose form**: Handles both create (memo is None) and edit (memo is Some).

**Pattern matching everywhere**:
```html
{% match memo %}
    {% when Some with (m) %}
        {# Edit mode - prefill fields #}
        value="{{ m.title }}"
    {% when None %}
        {# Create mode - empty fields #}
{% endmatch %}
```

**Nested pattern matching**:
```html
{% match memo %}
    {% when Some with (m) %}
        {% match m.description %}
            {% when Some with (desc) %}{{ desc }}{% when None %}
        {% endmatch %}
    {% when None %}
{% endmatch %}
```

Safely handles `Option<Option<String>>` for optional description field.

**Method calls on template values**:
```html
value="{{ m.date_to_local_format() }}"
```

Askama allows calling methods on template values. This formats DateTime for HTML5 datetime-local input (Chapter 11).

### Step 10: Create Error Page Template

Create `templates/pages/error.html`:

```html
{% extends "base.html" %}

{% block title %}Error - Memos App{% endblock %}

{% block content %}
<div class="container">
    <div class="error-page">
        <h2>Oops! Something went wrong</h2>
        <p class="error-message">{{ message }}</p>
        <p class="error-details">{{ details }}</p>
        <a href="/" class="btn btn-primary">Back to Home</a>
    </div>
</div>
{% endblock %}
```

Simple error page extending base layout, showing error message and details with a link back to home.

### Step 11: Create Template Structs in Rust

Now that all HTML templates exist, create Rust structs that map to them. These structs tell Askama which template file to use and what data to provide.

Add template structs to `src/handlers/web.rs`. If the file doesn't exist yet, create it:

```rust
use actix_web::{web, HttpResponse};
use askama::Template;
use crate::dto::memo_dto::MemoResponseDto;

/// Homepage template displaying list of memos
///
/// Maps to templates/pages/index.html
#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    pub memos: Vec<MemoResponseDto>,
}

/// Memo list component for AJAX updates
///
/// Maps to templates/components/memo_list.html
#[derive(Template)]
#[template(path = "components/memo_list.html")]
pub struct MemoListTemplate {
    pub memos: Vec<MemoResponseDto>,
}

/// Single memo item for AJAX updates
///
/// Maps to templates/components/memo_item.html
#[derive(Template)]
#[template(path = "components/memo_item.html")]
pub struct MemoItemTemplate {
    pub memo: MemoResponseDto,
}

/// Memo form component (create and edit)
///
/// Maps to templates/components/memo_form.html
/// memo is None for create, Some for edit
#[derive(Template)]
#[template(path = "components/memo_form.html")]
pub struct MemoFormTemplate {
    pub memo: Option<MemoResponseDto>,
}

/// Error page template
///
/// Maps to templates/pages/error.html
#[derive(Template)]
#[template(path = "pages/error.html")]
pub struct ErrorTemplate {
    pub message: String,
    pub details: String,
}
```

**Key points**:

**Template derive macro**:
```rust
#[derive(Template)]
```
Tells Askama to generate template rendering code for this struct.

**Template path attribute**:
```rust
#[template(path = "pages/index.html")]
```
Specifies which template file to use, relative to `templates/` directory.

**Struct fields must match template usage**:
```rust
pub struct IndexTemplate {
    pub memos: Vec<MemoResponseDto>,  // Template uses {{ memos }}
}
```

If template references `{{ memos.len() }}` but struct doesn't have `memos` field, **compilation fails**.

**Option handling**:
```rust
pub memo: Option<MemoResponseDto>,
```
Templates use `{% match memo %}{% when Some %}...{% when None %}...{% endmatch %}` to handle both states.

### Step 12: Register Web Handler Module

If you haven't already, add the web handler module to `src/handlers/mod.rs`:

```rust
pub mod health;
pub mod memos;
pub mod web;
```

This makes template structs and web handlers available throughout the application. The template structs live in the same file as the handlers that use them for better cohesion.

### Step 13: Add Date Formatting Helper to DTO

The memo form template calls `date_to_local_format()` on memo objects for HTML5 datetime-local inputs. Add this method to `MemoResponseDto`.

Edit `src/dto/memo_dto.rs`, add this method to the `impl MemoResponseDto` block:

```rust
impl MemoResponseDto {
    /// Format date for HTML datetime-local input (YYYY-MM-DDThh:mm format)
    pub fn date_to_local_format(&self) -> String {
        self.date_to.format("%Y-%m-%dT%H:%M").to_string()
    }
}
```

**Why this is needed**:
- HTML5 `<input type="datetime-local">` requires format: `2025-01-15T14:30`
- Chrono's default display format doesn't match
- This method converts DateTime to the exact format HTML expects
- Called directly in templates: `{{ m.date_to_local_format() }}`

### Step 14: Verify Compilation

Test that Askama successfully compiles all templates:

```bash
cargo build
```

**What happens during build**:

1. **Askama reads templates/**: Finds all .html files referenced by template structs
2. **Parses template syntax**: Validates Jinja2-like syntax in each template
3. **Type checks expressions**: Verifies `{{ memo.title }}` matches `MemoResponseDto::title`
4. **Checks control flow**: Validates all `{% if %}`, `{% for %}`, `{% match %}` expressions
5. **Verifies includes**: Ensures all `{% include %}` and `{% extends %}` paths exist
6. **Generates Rust code**: Creates optimized rendering functions
7. **Compiles into binary**: Templates become part of executable (no runtime loading)

**Expected output**:
```
   Compiling actix-web-template v0.2.1
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

**If you see errors**:
- **"template not found"**: Check `#[template(path = "...")]` matches file location
- **"field not found"**: Template uses field not in struct
- **"method not found"**: Template calls method not on type (check `date_to_local_format`)
- **"syntax error"**: Invalid Askama syntax in template

All template errors are caught now, before any code runs.

### Step 15: Test Template Rendering (Optional)

Create a simple test to verify templates render correctly.

Add to the end of `src/handlers/web.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_index_template_renders() {
        let template = IndexTemplate {
            memos: vec![
                MemoResponseDto {
                    id: Uuid::new_v4(),
                    title: "Test Memo".to_string(),
                    description: Some("Test description".to_string()),
                    date_to: Utc::now(),
                    completed: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ],
        };

        let rendered = template.render().expect("template should render");
        assert!(rendered.contains("Test Memo"));
        assert!(rendered.contains("Test description"));
    }

    #[test]
    fn test_error_template_renders() {
        let template = ErrorTemplate {
            message: "Not Found".to_string(),
            details: "The requested resource was not found".to_string(),
        };

        let rendered = template.render().expect("template should render");
        assert!(rendered.contains("Not Found"));
        assert!(rendered.contains("The requested resource was not found"));
    }

    #[test]
    fn test_memo_form_create_mode() {
        let template = MemoFormTemplate { memo: None };
        let rendered = template.render().expect("template should render");
        assert!(rendered.contains("New Memo"));
        assert!(rendered.contains("Create"));
    }

    #[test]
    fn test_memo_form_edit_mode() {
        let memo = MemoResponseDto {
            id: Uuid::new_v4(),
            title: "Edit Test".to_string(),
            description: None,
            date_to: Utc::now(),
            completed: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let template = MemoFormTemplate { memo: Some(memo) };
        let rendered = template.render().expect("template should render");
        assert!(rendered.contains("Edit Memo"));
        assert!(rendered.contains("Update"));
        assert!(rendered.contains("Edit Test"));
    }
}
```

Run tests:

```bash
cargo test handlers::web::tests
```

**What these tests verify**:
- Templates compile (implicit - test wouldn't run if template had syntax errors)
- Templates render without runtime errors
- Variable interpolation works (title appears in output)
- Conditional rendering works (create vs edit mode)
- Pattern matching works (Some/None handling)

These tests catch issues like missing fields or incorrect template logic.

## Checkpoint

At this point, you should have:

**Directory structure**:
```
templates/
├── base.html
├── pages/
│   ├── index.html
│   └── error.html
├── components/
│   ├── memo_list.html
│   ├── memo_item.html
│   └── memo_form.html
└── partials/
    ├── header.html
    └── footer.html
```

**Rust code**:
- `src/handlers/web.rs` with 5 template structs
- `date_to_local_format()` method on MemoResponseDto
- Web module registered in handlers/mod.rs

**Verification**:
```bash
# Templates compile without errors
cargo build

# Tests pass
cargo test templates::tests
```

**What you can do**:
- Templates compile into your binary (no runtime template loading)
- Type-safe rendering (compiler catches template errors)
- All templates inherit from base.html for consistent layout
- Components are reusable across pages and AJAX updates

**What you cannot do yet**:
- Render templates in HTTP handlers (Chapter 11)
- Serve static CSS (Chapter 12)
- Use templates in actual web pages (Chapter 11)

The templates exist and compile, but aren't connected to web handlers yet. That's the next chapter.

## Common Issues and Solutions

### Issue: Template not found error

**Symptom**:
```
error: template not found: pages/index.html
```

**Cause**: Template path in `#[template(path = "...")]` doesn't match actual file location.

**Solution**:
```bash
# Check templates directory exists
ls -R templates/

# Verify exact path matches attribute
# If file is templates/pages/index.html, use:
#[template(path = "pages/index.html")]  # ✓ Correct
#[template(path = "index.html")]        # ✗ Wrong
```

### Issue: Field not found in template struct

**Symptom**:
```
error: no field `title` on type `&MemoResponseDto`
   --> templates/components/memo_item.html:3
    |
3   |     <h3>{{ memo.title }}</h3>
    |                 ^^^^^
```

**Cause**: Template uses field that doesn't exist on the struct.

**Solution**:
1. Check DTO definition: `pub struct MemoResponseDto { pub title: String, ... }`
2. Ensure field is public
3. Verify field name matches exactly (case-sensitive)
4. Check you're using correct struct type in template struct

### Issue: Method not found error

**Symptom**:
```
error: no method named `date_to_local_format` found for type `DateTime<Utc>`
```

**Cause**: Template calls method that doesn't exist.

**Solution**:
Add method to impl block in `src/dto/memo_dto.rs`:
```rust
impl MemoResponseDto {
    pub fn date_to_local_format(&self) -> String {
        self.date_to.format("%Y-%m-%dT%H:%M").to_string()
    }
}
```

Methods called in templates must be public and return types that Askama can display.

### Issue: Invalid syntax in template

**Symptom**:
```
error: failed to parse template
   --> templates/components/memo_item.html:5
    |
5   | {% fi memo.completed %}
    |    ^^ expected 'if' or 'for' or 'match'
```

**Cause**: Typo or invalid Askama syntax.

**Solution**:
- Check syntax against Askama documentation
- Common mistakes:
  - `{% fi %}` → `{% if %}`
  - `{{ for memo }}` → `{% for memo in memos %}`
  - Missing `{% endif %}` or `{% endfor %}`
  - Wrong delimiter: `{{ if ... }}` → `{% if ... %}`

### Issue: Include path not found

**Symptom**:
```
error: template not found: components/memo_item.html
```

**Cause**: `{% include "components/memo_item.html" %}` references file that doesn't exist.

**Solution**:
```bash
# Create the missing template
touch templates/components/memo_item.html

# Or fix the path in the include statement
{% include "components/memo_item.html" %}  # Must match actual file path
```

### Issue: Pattern match not exhaustive

**Symptom**:
```
error: match expression is not exhaustive
   --> templates/components/memo_form.html:10
```

**Cause**: Pattern matching doesn't cover all cases.

**Solution**:
Always handle both Some and None for Option:
```html
{% match memo %}
    {% when Some with (m) %}{{ m.title }}{% endmatch %}
    {% when None %}{% endmatch %}
{% endmatch %}
```

Even if None does nothing, you must include it.

### Issue: Nested includes not working

**Symptom**: Included template doesn't render or throws error.

**Cause**: Askama resolves includes relative to `templates/` directory.

**Solution**:
```html
<!-- In templates/pages/index.html -->
{% include "components/memo_list.html" %}  # ✓ Correct (from templates/)
{% include "../components/memo_list.html" %} # ✗ Wrong (no relative paths)
```

All paths are from `templates/` root, never use `../`.

## Code Review

Now that all templates are created, let's review the complete implementation.

### Principles Demonstrated

**Separation of Concerns**
- **Templates** handle presentation (HTML structure, styling classes)
- **Rust structs** provide type-safe data
- **Business logic** stays in service layer (not in templates)

**Type Safety**
- Compile-time checking prevents runtime template errors
- Mismatched fields or types caught during `cargo build`
- Refactoring Rust types automatically flags outdated templates

**Security by Default**
- All variables automatically HTML-escaped
- XSS prevention without developer intervention
- Safe by default, unsafe only when explicitly marked `|safe`

**Performance**
- Zero runtime template parsing overhead
- Templates compiled directly into binary
- Rendering is simple string concatenation (extremely fast)

**Reusability**
- Base layout shared across all pages
- Components (memo_item, memo_list) reused in multiple contexts
- Partials (header, footer) included everywhere

### Architecture Review

**Template hierarchy**:
```
base.html
├── pages/index.html (full page)
│   └── includes components/memo_list.html
│       └── includes components/memo_item.html
├── pages/error.html (full page)
└── includes partials/header.html, footer.html
```

**Data flow**:
1. **Handler** creates template struct with data
2. **Askama** renders template using compiled code
3. **Template** outputs HTML string
4. **Actix** sends HTML to browser

**AJAX partial updates**:
- Full pages use pages/ templates (complete HTML documents)
- AJAX responses use components/ templates (HTML fragments)
- Same components render in both contexts (initial load + updates)

### Structure Review

**Base template (`base.html`)**:
- DOCTYPE and HTML structure
- Links to static CSS (Chapter 12)
- Includes header/footer (consistent navigation)
- Defines blocks for child templates (title, head_scripts, content)

**Page templates** (`pages/*.html`):
- Extend base.html
- Override title block
- Add page-specific JavaScript in head_scripts block
- Provide main content in content block

**Component templates** (`components/*.html`):
- Standalone HTML fragments
- Reusable in pages and AJAX responses
- Accept data via template struct fields
- Use control flow (if, for, match)

**Partial templates** (`partials/*.html`):
- Small, reusable pieces
- No data dependencies (or minimal)
- Included via `{% include %}`

**Rust template structs**:
- Map 1-to-1 with template files
- Derive Template trait
- Specify template path
- Fields match template variable usage

### Testing Review

**What we test**:
- Templates render without errors (implicit via `cargo build`)
- Variable interpolation works (content appears in output)
- Conditional rendering (create vs edit mode)
- Option handling (Some/None pattern matching)

**What we don't test** (yet):
- Integration with handlers (Chapter 11)
- User interactions (JavaScript) (Chapter 11)
- CSS styling (Chapter 12)
- Full end-to-end flows (Chapter 14)

Template rendering tests are cheap (fast, no database) and catch issues early.

## Testing

### Manual Testing

**Step 1**: Verify templates compile:
```bash
cargo build
```

Expected: No errors. If templates have syntax issues, build fails here.

**Step 2**: Run template unit tests:
```bash
cargo test handlers::web::tests
```

Expected:
```
running 4 tests
test handlers::web::tests::test_index_template_renders ... ok
test handlers::web::tests::test_error_template_renders ... ok
test handlers::web::tests::test_memo_form_create_mode ... ok
test handlers::web::tests::test_memo_form_edit_mode ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Step 3**: Inspect generated output (optional):
```rust
// Add to a test
let rendered = template.render().unwrap();
println!("{}", rendered);
```

Run with `cargo test -- --nocapture` to see HTML output.

### What to Test

**Template rendering**:
```rust
let template = IndexTemplate { memos: vec![] };
assert!(template.render().is_ok());
```

**Variable interpolation**:
```rust
let template = ErrorTemplate {
    message: "Test Error".to_string(),
    details: "Details here".to_string(),
};
let html = template.render().unwrap();
assert!(html.contains("Test Error"));
assert!(html.contains("Details here"));
```

**Conditional rendering**:
```rust
// Create mode (memo is None)
let template = MemoFormTemplate { memo: None };
let html = template.render().unwrap();
assert!(html.contains("New Memo"));
assert!(html.contains("Create"));

// Edit mode (memo is Some)
let memo = MemoResponseDto { /* ... */ };
let template = MemoFormTemplate { memo: Some(memo) };
let html = template.render().unwrap();
assert!(html.contains("Edit Memo"));
assert!(html.contains("Update"));
```

**Option handling**:
```rust
let memo = MemoResponseDto {
    title: "Test".to_string(),
    description: Some("Has description".to_string()),
    // ...
};
let template = MemoItemTemplate { memo };
let html = template.render().unwrap();
assert!(html.contains("Has description"));

let memo_no_desc = MemoResponseDto {
    title: "Test".to_string(),
    description: None,
    // ...
};
let template = MemoItemTemplate { memo: memo_no_desc };
let html = template.render().unwrap();
// Description section should not appear
assert!(!html.contains("memo-description"));
```

### Integration Testing

Full integration tests (rendering templates in actual HTTP handlers) come in Chapter 11 after web handlers are implemented. For now, unit tests verify template logic works correctly.

## Summary

You've successfully implemented a complete server-side template system using Askama:

**Key achievements**:
1. **Template infrastructure**: Created organized template directory structure (pages, components, partials)
2. **Base layout**: Built reusable base template with header, footer, and content blocks
3. **Component architecture**: Created reusable memo components (list, item, form)
4. **Type-safe structs**: Defined Rust template structs with compile-time validation
5. **Control flow**: Used if/for/match for dynamic rendering
6. **Option handling**: Safely handled Option types with pattern matching
7. **XSS protection**: Automatic HTML escaping prevents cross-site scripting
8. **Testing**: Verified templates compile and render correctly

**What makes this approach powerful**:
- **Compile-time safety**: Template errors caught during build, not at runtime
- **Zero overhead**: Templates compiled into binary, no runtime parsing
- **Type safety**: Rust's type system extends into templates
- **Performance**: Rendering is optimized string concatenation
- **Maintainability**: Refactoring Rust code automatically flags template issues

**How this fits into the application**:
- Templates define the presentation layer (View in MVC)
- Template structs bridge handlers (Controller) and templates (View)
- DTOs provide the data model (Model) consumed by templates
- Handlers (Chapter 11) will create template structs and return rendered HTML
- JavaScript enhances templates with interactivity (already in index.html)

Templates are ready. Next chapter connects them to HTTP handlers.

## Next Steps

In **Chapter 11: Web Page Handlers**, you'll:
- Create HTTP handlers that render these templates
- Handle HTML form submissions (POST, PUT, DELETE)
- Implement the "Redirect After Post" pattern
- Return AJAX-friendly HTML fragments
- Integrate templates with the service layer
- Build the complete web UI for memo management

The templates exist and compile. Now it's time to serve them over HTTP and make the application interactive.

## Additional Resources

### Official Documentation
- [Askama Documentation](https://djc.github.io/askama/) - Complete guide to template syntax and features
- [Askama GitHub](https://github.com/djc/askama) - Source code and examples
- [Askama Book](https://djc.github.io/askama/askama.html) - In-depth tutorials

### Template Syntax
- [Jinja2 Template Designer Documentation](https://jinja.palletsprojects.com/en/3.1.x/templates/) - Askama syntax is based on Jinja2
- [Template Inheritance](https://jinja.palletsprojects.com/en/3.1.x/templates/#template-inheritance) - Understanding extends and blocks
- [Control Structures](https://jinja.palletsprojects.com/en/3.1.x/templates/#list-of-control-structures) - if, for, match

### Related Topics
- [Server-Side Rendering vs Client-Side Rendering](https://web.dev/rendering-on-the-web/) - Understand trade-offs
- [Progressive Enhancement](https://developer.mozilla.org/en-US/docs/Glossary/Progressive_Enhancement) - Build for all users
- [XSS Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html) - OWASP XSS guide

### Alternative Template Engines
- [Tera](https://tera.netlify.app/) - Runtime template engine (Jinja2-like)
- [Handlebars-Rust](https://github.com/sunng87/handlebars-rust) - Handlebars for Rust
- [Maud](https://maud.lambda.xyz/) - HTML templates in Rust syntax
