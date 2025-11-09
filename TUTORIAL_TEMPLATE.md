# Tutorial Chapter Template

This document defines the formal structure, style, and content guidelines for all tutorial chapters in this project.

## Purpose

This template ensures:
- Consistent learning experience across all chapters
- Predictable structure for students
- Quality standards for content
- Maintainability and extensibility

## Reference Implementation

**Chapter 07: Service Layer** serves as the reference implementation that fully embodies this template.

---

## Chapter Structure

### Required Sections (In Order)

All chapters **MUST** include these sections in this exact order:

1. **Title Header** (H1)
2. **Overview**
3. **Prerequisites**
4. **Learning Objectives**
5. **Concepts Covered**
6. **Step-by-Step Instructions**
7. **Checkpoint**
8. **Common Issues and Solutions**
9. **Code Review**
10. **Testing** (if applicable)
11. **Summary**
12. **Next Steps**
13. **Additional Resources** (optional)

### Section Definitions

---

#### 1. Title Header

```markdown
# Chapter N: [Descriptive Title]
```

**Requirements:**
- Use chapter number (0-padded for 0-9)
- Title should clearly describe the chapter's focus
- Use title case

**Examples:**
- `# Chapter 0: Prerequisites and Environment Setup`
- `# Chapter 7: Service Layer - Business Logic and Transactions`
- `# Chapter 15: Docker Deployment`

---

#### 2. Overview

```markdown
## Overview

[2-3 paragraphs explaining what this chapter covers and why it matters]
```

**Content Guidelines:**
- **Paragraph 1:** What you'll build in this chapter
- **Paragraph 2:** How it fits into the overall architecture
- **Paragraph 3:** Key benefits or learning outcomes

**Length:** 50-150 words

**Tone:** Motivational and clear

**Example:**
```markdown
## Overview

The service layer sits between your handlers (HTTP layer) and repositories
(data access layer), orchestrating business logic, coordinating transactions,
and transforming data between DTOs and entities. This layer encapsulates
domain rules, sanitizes user input, and ensures data consistency across
multiple operations.

In this chapter, you'll build a complete service layer that handles business
logic, coordinates database transactions, and provides a clean API for your
handlers to use.
```

---

#### 3. Prerequisites

```markdown
## Prerequisites

### Completed
- Chapter N-1: [Previous Chapter Title]
- Chapter N-2: [Other Required Chapter]

### Required Knowledge
- [Concept 1]
- [Concept 2]

### Required Software
- [Tool 1] (version)
- [Tool 2] (version)
```

**Content Guidelines:**
- List all previous chapters that must be completed
- Specify Rust concepts student should understand
- List any software/tools needed for this chapter
- Include version numbers where relevant

**Must Include:**
- Previously completed chapters
- Conceptual prerequisites
- Software prerequisites (if any)

---

#### 4. Learning Objectives

```markdown
## Learning Objectives

By the end of this chapter, you will:
- [Specific skill or understanding 1]
- [Specific skill or understanding 2]
- [Specific skill or understanding 3]
- [Specific skill or understanding 4]
- [Specific skill or understanding 5]
```

**Content Guidelines:**
- Use "you will" (future tense, active voice)
- 5-7 concrete, measurable objectives
- Start each with action verbs: understand, build, implement, create, test
- Be specific and achievable within the chapter
- Focus on skills, not just knowledge

**Example:**
```markdown
By the end of this chapter, you will:
- Understand the service layer's role in layered architecture
- Implement DTO to Entity conversions
- Sanitize user input to prevent XSS attacks
- Coordinate transactions across multiple operations
- Apply business logic validation
- Create a clean service API for handlers
- Test service layer logic independently
```

---

#### 5. Concepts Covered

```markdown
## Concepts Covered

### [Main Concept 1]

[2-4 paragraphs explaining the concept]

**Why [this approach]?**

[Bullet points explaining benefits]

### [Main Concept 2]

[Explanation with code examples if needed]
```

**Content Guidelines:**
- 3-5 major concepts per chapter
- Each concept gets a subsection (H3)
- Include "Why?" explanations
- Use diagrams where helpful (ASCII art or descriptions)
- Provide context before diving into code
- Compare with alternatives when relevant

**Required Elements:**
- Conceptual explanation (what)
- Rationale (why)
- Practical context (when/where)

**Example Structure:**
```markdown
### The Service Layer Pattern

The service layer pattern separates business logic from HTTP concerns
and database operations:

[ASCII diagram showing layers]

**Why separate services from repositories?**

1. **Single Responsibility**: Repositories handle data access; services handle business logic
2. **Transaction Coordination**: Services orchestrate multiple repository calls
3. **Reusability**: Business logic can be reused across different handlers
```

---

#### 6. Step-by-Step Instructions

```markdown
## Step-by-Step Instructions

### Step 1: [Action to Perform]

**Why**: [Explanation of purpose]

**How**: [Instructions]

[Code blocks]

**Verify**:
```bash
[Commands to verify this step works]
```

[Expected output]

---

### Step 2: [Next Action]

[Same structure as Step 1]
```

**Content Guidelines:**
- Number steps sequentially starting from 1
- Each step must include: Why, How, Verify
- Steps should build progressively
- Include complete, copy-pasteable code
- Use horizontal rule (---) between steps
- Show expected output after verification

**Code Block Standards:**
- Use language tags: ```rust, ```bash, ```toml, etc.
- No indentation before code fences
- Include file paths in comments when relevant
- **Show only NEW or UPDATED code**, not entire files
- Use comments to indicate where code goes: `// ... existing code ...`
- Show just enough context to locate the change

**Verification:**
- Every step must be verifiable
- Provide exact commands to run
- Show expected output
- Include error checking

**Example:**
```markdown
### Step 1: Add Dependencies

**Why**: We need the `ammonia` crate for HTML sanitization.

**How**: Add to `Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
ammonia = "4.0"
```

**Verify**:
```bash
cargo build
```

You should see ammonia being compiled in the output.

---

### Step 2: Create Sanitization Function

**Why**: Centralize HTML sanitization logic.

**How**: Add to `src/utils/sanitize.rs`:

```rust
use ammonia::clean;

pub fn sanitize_html(input: &str) -> String {
    clean(input)
}
```

**Note**: Only show the NEW function being added, not the entire file.
```

---

#### 7. Checkpoint

```markdown
## Checkpoint

At this point, you should have:

1. [Specific deliverable 1]
2. [Specific deliverable 2]
3. [Specific deliverable 3]

**Verify everything works**:

```bash
# Run all tests
cargo test

# Build the project
cargo build

# Check for warnings
cargo clippy
```

Expected output:
```
[Show expected output]
```
```

**Content Guidelines:**
- List concrete deliverables
- Provide comprehensive verification commands
- Show expected output
- This is a "pause and check" moment
- Student should be able to verify 100% completion

**Required:**
- Checklist of what should be complete
- Commands to verify each item
- Expected output examples

---

#### 8. Common Issues and Solutions

```markdown
## Common Issues and Solutions

### Issue: [Problem Description]

**Symptoms**: [What the student sees]

**Cause**: [Why it happens]

**Solution**:
```[language]
[How to fix it]
```

---

### Issue: [Another Problem]

[Same structure]
```

**Content Guidelines:**
- Include 3-5 common issues
- Use actual problems students encounter
- Provide clear symptoms (error messages, behaviors)
- Explain the root cause
- Give complete solutions
- Use horizontal rule between issues

**Issue Categories:**
- Compilation errors
- Runtime errors
- Configuration problems
- Environment issues
- Logic/behavior problems

**Format:**
- **Symptoms**: Exact error message or behavior
- **Cause**: Technical explanation
- **Solution**: Step-by-step fix with code

---

#### 9. Code Review

```markdown
## Code Review

### Key Design Principles Demonstrated

1. **[Principle 1]**: [Explanation of how code demonstrates this]
2. **[Principle 2]**: [Explanation]
3. **[Principle 3]**: [Explanation]

### Architecture Benefits

[Diagram or explanation showing how this fits in overall architecture]

```
┌─────────────────────────────────────────┐
│ [Show architectural diagram]            │
│ [Highlight how new component fits]      │
└─────────────────────────────────────────┘
```

### Complete [Component] Structure

Your [component] should now look like this:

```
[Directory tree showing file structure]
```

**Content Guidelines:**
- **Start with principles**: Explain design patterns BEFORE showing code
- **Show architecture**: Context and benefits come BEFORE implementation
- **End with structure**: Directory tree is the concrete result
- Highlight best practices demonstrated
- Connect to broader architecture
- Point out how this chapter advances the overall design

**Required Elements (in order):**
1. Design principles explained
2. Architecture context and benefits
3. File/directory structure

---

#### 10. Testing

```markdown
## Testing

### Unit Test Coverage

The [component] tests cover:

- ✅ [Feature 1]
- ✅ [Feature 2]
- ✅ [Feature 3]

### Manual Testing

[If applicable, provide manual testing steps]

### Running Tests

```bash
# Run specific test file
cargo test --test [test_file]

# Run all tests
cargo test
```

[Show expected output]
```

**Content Guidelines:**
- List what tests cover
- Provide test running commands
- Show expected output
- Include manual testing if applicable
- Explain what each test validates

**When to Include:**
- Include if the chapter adds testable code
- Skip for pure setup chapters (Chapter 0)
- Skip if testing is covered in another chapter

---

#### 11. Summary

```markdown
## Summary

### What You Learned

In this chapter, you:

1. **[Major accomplishment 1]**: [Brief description]
2. **[Major accomplishment 2]**: [Brief description]
3. **[Major accomplishment 3]**: [Brief description]

### Architecture Progress

You've now completed the [layer/component]:

```
[Show architecture diagram with completed parts highlighted]
```

### Key Takeaways

1. **[Principle 1]**: [Core lesson]
2. **[Principle 2]**: [Core lesson]
3. **[Principle 3]**: [Core lesson]
```

**Content Guidelines:**
- Recap major accomplishments (past tense)
- Show architectural progress
- Highlight 3-5 key takeaways
- Connect to broader learning goals
- Celebrate completion

**Required Elements:**
- What you learned (concrete)
- Where you are architecturally
- Key principles/takeaways

---

#### 12. Next Steps

```markdown
## Next Steps

### Required: Chapter N+1 - [Next Chapter Title]

[1-2 sentences describing what comes next and how it builds on this chapter]

### Optional Exercises

Before moving on, try these challenges:

1. **[Challenge 1]**: [Brief description]
2. **[Challenge 2]**: [Brief description]
3. **[Challenge 3]**: [Brief description]
```

**Content Guidelines:**
- Keep it SHORT - 1-2 sentences for next chapter preview
- Don't list detailed bullet points
- Focus on the connection: "Now that you have X, next you'll build Y"
- Optional exercises should be brief one-liners
- No code examples in this section

**Required:**
- Link to next chapter title
- Brief 1-2 sentence preview

**Optional:**
- 2-3 short exercise suggestions (one line each)

**Example:**
```markdown
## Next Steps

### Required: Chapter 8 - REST API Handlers

Now that you have a complete service layer, you'll create HTTP handlers that accept requests, call service methods, and return JSON responses.

### Optional Exercises

Before moving on, try these challenges:

1. **Add a business rule**: Prevent updates to completed memos
2. **Implement soft deletes**: Add a `deleted_at` field instead of hard deletes
3. **Add a search method**: Search memos by title or description
```

---

#### 13. Additional Resources (Optional)

```markdown
## Additional Resources

### [Topic Category 1]
- [Resource title with link](URL) - Brief description
- [Resource title with link](URL) - Brief description

### [Topic Category 2]
- [Resource title with link](URL) - Brief description

### [Topic Category 3]
- [Resource title with link](URL) - Brief description
```

**Content Guidelines:**
- Organize by topic category
- Provide official documentation links
- Include blog posts or articles
- Add video tutorials if available
- Keep descriptions brief (one line)

**Resource Types:**
- Official documentation
- Blog posts
- Video tutorials
- Books
- Related tools

**Quality Standards:**
- Authoritative sources
- Up-to-date content
- Relevant to chapter topic
- Free/accessible resources preferred

---

## Writing Style Guidelines

### Voice and Tone

- **Person**: Second person ("you will", "you'll create")
- **Tense**:
  - Future for objectives ("you will learn")
  - Present for explanations ("services coordinate")
  - Imperative for instructions ("Add to Cargo.toml")
  - Past for summary ("you learned")
- **Tone**: Professional but approachable, encouraging
- **Technical Level**: Intermediate (assumes basic Rust knowledge)

### Formatting Standards

#### Headers

```markdown
# Chapter Title (H1) - Only at the very start
## Major Section (H2) - Main sections
### Subsection (H3) - Within major sections
#### Sub-subsection (H4) - Specific details within subsections
```

**Rules:**
- Only ONE H1 per chapter (the title)
- Use H2 for the 11 required sections
- Use H3 for steps and subsections
- Use H4 sparingly for detailed breakdowns
- Never skip levels (H2 → H4)

#### Code Blocks

```markdown
# Always specify language
```rust
// Rust code
```

# Use appropriate language tags
```bash
# Shell commands
```

```toml
# Configuration files
```

```sql
# SQL queries
```
```

**Rules:**
- Always include language tag
- No indentation before triple backticks
- Include file paths as comments when relevant
- Show complete context (imports, etc.)

#### Lists

```markdown
# Unordered lists
- Item 1
- Item 2
  - Sub-item (2 spaces)

# Ordered lists
1. First item
2. Second item
   - Sub-point (3 spaces)

# Checklists
- [ ] Not done
- [x] Completed
```

#### Emphasis

```markdown
**Bold** for emphasis, key terms, UI elements
*Italic* for light emphasis, foreign terms
`code` for inline code, commands, file names
```

#### Links

```markdown
[Link text](URL)
[Chapter 8: REST API Handlers](chapter-08.md)
```

#### Horizontal Rules

```markdown
---
```

Use between:
- Steps
- Issues in Common Issues section
- Major concept sections

### Code Standards

#### Example Code

- **Show only NEW or CHANGED code** - Don't repeat entire files
- Include necessary imports for the NEW code
- Use `// ... existing code ...` to indicate omitted sections
- Provide just enough context to locate where code goes
- Match template code exactly
- Follow Rust conventions

**Good Example:**
```rust
// src/services/memo_service.rs

impl MemoService {
    // ... existing methods ...

    /// NEW: Creates multiple memos in a transaction
    pub async fn create_memos_batch(&self, dtos: Vec<CreateMemoDto>) -> Result<Vec<MemoResponseDto>, AppError> {
        // Implementation here
    }
}
```

**Bad Example:**
```rust
// src/services/memo_service.rs - DON'T show entire file

use sea_orm::DatabaseConnection;
// ... 200 lines of existing code ...
pub struct MemoService { /* ... */ }
impl MemoService {
    pub fn new() { /* ... */ }
    pub async fn get_all() { /* ... */ }
    // ... all existing methods ...
    pub async fn create_memos_batch() { /* NEW but buried */ }
}
```

#### Comments in Code

```rust
// Brief explanations for complex logic
// File path indicators when showing snippets
// Not needed for obvious code
```

**Rules:**
- Comment the "why", not the "what"
- Keep comments concise
- Use `//` for inline, `///` for docs
- No emoji in code comments

### Content Guidelines

#### Acronyms

- Define on first use: "Data Transfer Object (DTO)"
- Use abbreviation thereafter
- Common acronyms: API, HTTP, SQL, CRUD, DTO, ORM

#### Technical Terms

- Define when first introduced
- Use consistently throughout
- Bold on first definition: **service layer**

#### Examples

- Provide concrete examples
- Use the memo domain consistently
- Show both correct and incorrect approaches
- Explain why one is better

#### Diagrams

- Use ASCII art for simple diagrams
- Keep diagrams simple and focused
- Explain components clearly
- Show data flow when relevant

```
Example ASCII diagram:
┌──────────────┐
│   Handlers   │
└──────┬───────┘
       │
┌──────▼───────┐
│   Service    │
└──────┬───────┘
       │
┌──────▼───────┐
│  Repository  │
└──────────────┘
```

---

## Quality Checklist

Before considering a chapter complete, verify:

### Structure
- [ ] All required sections present
- [ ] Sections in correct order
- [ ] H1 used only for title
- [ ] Proper heading hierarchy

### Content
- [ ] Overview explains what and why
- [ ] Prerequisites clearly listed
- [ ] 5-7 learning objectives
- [ ] Concepts explained before code
- [ ] Step-by-step instructions complete
- [ ] Each step has Why/How/Verify
- [ ] Checkpoint validates completion
- [ ] 3-5 common issues covered
- [ ] Code review explains design
- [ ] Testing section (if applicable)
- [ ] Summary recaps learning
- [ ] Next steps preview next chapter

### Code
- [ ] All code examples complete
- [ ] Language tags on code blocks
- [ ] Code matches template exactly
- [ ] Verification commands provided
- [ ] Expected output shown

### Style
- [ ] Second person ("you will")
- [ ] Consistent terminology
- [ ] Professional tone
- [ ] No emoji (except in examples if needed)
- [ ] Proper formatting

### Testing
- [ ] Chapter builds successfully
- [ ] All commands work as written
- [ ] Tests pass
- [ ] Links are valid
- [ ] No typos or grammar errors

---

## Metrics

### Target Ranges

- **Length**: 1,000-1,500 lines
- **Code blocks**: 50-150
- **Steps**: 4-8 major steps
- **Reading time**: 20-40 minutes
- **Completion time**: 1-3 hours (including coding)

### Complexity

- **Beginner-friendly**: Chapters 0-4
- **Intermediate**: Chapters 5-12
- **Advanced**: Chapters 13-18

---

## Examples by Chapter Type

### Setup Chapter (Chapter 0)
- May omit: Concepts Covered, Code Review, Testing
- Must have: Overview, Prerequisites, Learning Objectives, Steps, Checkpoint, Common Issues, Summary, Next Steps

### Code Chapter (Chapters 1-18)
- Must have: All sections
- Testing section required if code is testable
- Code Review shows architecture

### Integration Chapter (e.g., Docker, CI/CD)
- Focus on configuration and deployment
- May have fewer code blocks
- More emphasis on verification

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-06 | Initial template based on Chapter 07 analysis |

---

## Contributing

When updating this template:
1. Discuss changes with maintainers
2. Update version number
3. Test against existing chapters
4. Update all chapters if structure changes
5. Document rationale in version history

---

## Notes

This template is a living document. As we learn what works best for students, we'll refine it. Chapter 07 serves as the current reference implementation.
