use crate::model::*;

pub fn matches_filter(filter: &ExportFilter, setting: &ResolvedSetting) -> bool {
    match filter {
        ExportFilter::All => true,
        ExportFilter::Disabled => false,
        ExportFilter::Table(scope) => evaluate_table_filter(scope, setting),
    }
}

fn evaluate_table_filter(root: &FilterScope, setting: &ResolvedSetting) -> bool {
    let key_parts: Vec<&str> = setting.key.split('.').collect();

    // Rule 1: Exclusion Preferred Over Inclusion
    // Check if any tag exclusion matches
    for tag in &setting.tags {
        let tag_key = format!("#{}", tag);
        if let Some(FilterRule::Exclude) = root.rules.get(&tag_key) {
            return false;
        }
    }

    // Check if key or any ancestor prefix is excluded
    if is_key_excluded(root, &key_parts) {
        return false;
    }

    // Rule 7: Base Inclusion Scope
    // Check if filter is in Pure Exclusion Mode:
    // Pure exclusion mode: ONLY top-level exclusion rules and no child exclusions or any inclusions.
    if is_pure_exclusion_mode(root) {
        return true;
    }

    // Explicit Inclusion Mode: Must match an inclusion or an implicit parent inclusion
    // 1. Tag inclusion
    for tag in &setting.tags {
        let tag_key = format!("#{}", tag);
        if let Some(FilterRule::Include) = root.rules.get(&tag_key) {
            return true;
        }
    }

    // 2. Exact key, group, or implicit parent inclusion
    is_key_included(root, &key_parts)
}

fn is_key_excluded(scope: &FilterScope, parts: &[&str]) -> bool {
    if parts.is_empty() {
        return false;
    }

    let head = parts[0];
    let tail = &parts[1..];

    // Check if current part is explicitly excluded at this level
    if let Some(FilterRule::Exclude) = scope.rules.get(head) {
        return true;
    }

    // Check subscope
    if let Some(sub) = scope.subscopes.get(head)
        && is_key_excluded(sub, tail)
    {
        return true;
    }

    false
}

fn has_any_inclusions(scope: &FilterScope) -> bool {
    if scope.rules.values().any(|r| *r == FilterRule::Include) {
        return true;
    }
    for sub in scope.subscopes.values() {
        if has_any_inclusions(sub) {
            return true;
        }
    }
    false
}

fn has_child_exclusions(scope: &FilterScope) -> bool {
    // Child exclusions are exclusions in any subscope
    for sub in scope.subscopes.values() {
        if has_exclusions(sub) {
            return true;
        }
    }
    false
}

/// True if `scope` itself has an exclusion rule, or any of its descendants do.
fn has_exclusions(scope: &FilterScope) -> bool {
    scope.rules.values().any(|r| *r == FilterRule::Exclude) || has_child_exclusions(scope)
}

fn is_pure_exclusion_mode(root: &FilterScope) -> bool {
    !has_any_inclusions(root) && !has_child_exclusions(root)
}

fn is_key_included(scope: &FilterScope, parts: &[&str]) -> bool {
    if parts.is_empty() {
        return false;
    }

    let head = parts[0];
    let tail = &parts[1..];

    // Check if exact key or group inclusion exists at this level
    if let Some(FilterRule::Include) = scope.rules.get(head) {
        return true;
    }

    // Check subscope
    if let Some(sub) = scope.subscopes.get(head) {
        // If subscope has child exclusions and NO sibling inclusions in that subscope:
        // Rule 4: Implicit parent inclusion via child exclusion
        let has_incl_in_sub = has_any_inclusions(sub);
        let has_excl_in_sub = has_exclusions(sub);

        if !has_incl_in_sub && has_excl_in_sub {
            // Check if tail itself matches any exclusion inside sub
            if !is_key_excluded(sub, tail) {
                return true;
            }
        }

        if is_key_included(sub, tail) {
            return true;
        }
    }

    false
}
