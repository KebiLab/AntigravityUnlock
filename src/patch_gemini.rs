use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn run_gemini_patcher() -> Result<(), String> {
    let appdata = env::var("APPDATA").map_err(|_| "APPDATA not defined")?;

    let cli_dir = PathBuf::from(&appdata)
        .join("npm")
        .join("node_modules")
        .join("@google")
        .join("gemini-cli");
    let bundle_dir = cli_dir.join("bundle");

    let mut needs_reinstall = false;
    let mut is_patched = false;

    if bundle_dir.exists() && bundle_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&bundle_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "js") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("antigravity-vertex-api") || content.contains(r#"\"\""#)
                        {
                            needs_reinstall = true;
                            break;
                        }
                        if content.contains("projectId || \"\"") {
                            is_patched = true;
                        }
                    }
                }
            }
        }
    }

    if needs_reinstall {
        println!("Обнаружены старые патчи. Переустановка @google/gemini-cli...");
        if cli_dir.exists() {
            let _ = fs::remove_dir_all(&cli_dir);
        }
        let _ = Command::new("cmd")
            .args(["/C", "npm", "install", "-g", "@google/gemini-cli@latest"])
            .status();
    } else if is_patched {
        // Correct replacement found - only apply if some targets still remain
    }

    if !bundle_dir.exists() {
        return Err("Папка bundle не найдена в установке @google/gemini-cli".to_string());
    }

    // Define the patches
    let patches = vec![
        (
            "      return await this.requestPost(\n        \"loadCodeAssist\",\n        req\n      );",
            "      const res = await this.requestPost(\n        \"loadCodeAssist\",\n        req\n      );\n      if (res) {\n        if (!res.currentTier && res.ineligibleTiers && res.ineligibleTiers.length > 0) {\n          res.currentTier = { id: UserTierId.STANDARD, hasOnboardedPreviously: true };\n          delete res.ineligibleTiers;\n        }\n        if (!res.currentTier) {\n          res.currentTier = { id: UserTierId.STANDARD, hasOnboardedPreviously: true };\n        }\n        if (!res.paidTier) {\n          res.paidTier = { id: \"pro\", name: \"Pro\", availableCredits: [{ creditType: \"G1_CREDIT\", creditAmount: \"99999\" }] };\n        }\n      }\n      return res;"
        ),
        (
            "      } else if (isPermissionDeniedError(e2) && req.cloudaicompanionProject === \"cloudshell-gca\") {\n        throw new Error(\"Access to the default Cloud Shell Gemini project was denied.\\nPlease set your own Google Cloud project by running:\\ngcloud config set project [PROJECT_ID]\\nor setting export GOOGLE_CLOUD_PROJECT=...\");\n      } else {\n        throw e2;\n      }",
            "      } else if (isPermissionDeniedError(e2) || (Array.isArray(e2) && e2.length > 0 && (isPermissionDeniedError(e2[0]) || isPermissionDeniedError(e2[0]?.error)))) {\n        return {\n          currentTier: { id: UserTierId.STANDARD, hasOnboardedPreviously: true },\n          paidTier: { id: \"pro\", name: \"Pro\", availableCredits: [{ creditType: \"G1_CREDIT\", creditAmount: \"99999\" }] },\n          cloudaicompanionProject: req.cloudaicompanionProject || \"\"\n        };\n      } else {\n        throw e2;\n      }"
        ),
        (
            "  async listExperiments(metadata2) {\n    if (!this.projectId) {\n      throw new Error(\"projectId is not defined for CodeAssistServer.\");\n    }",
            "  async listExperiments(metadata2) {\n    if (!this.projectId) {\n      return { flags: [], experimentIds: [] };\n    }"
        ),
        (
            "    if (!loadRes.cloudaicompanionProject) {\n      if (projectId) {\n        return {\n          projectId,\n          userTier: loadRes.paidTier?.id ?? loadRes.currentTier.id ?? UserTierId.STANDARD,\n          userTierName: loadRes.paidTier?.name ?? loadRes.currentTier.name,\n          paidTier: loadRes.paidTier ?? void 0,\n          hasOnboardedPreviously: loadRes.currentTier.hasOnboardedPreviously ?? true\n        };\n      }\n      throwIneligibleOrProjectIdError(loadRes);\n    }",
            "    if (!loadRes.cloudaicompanionProject) {\n      return {\n        projectId: projectId || \"\",\n        userTier: loadRes.paidTier?.id ?? loadRes.currentTier.id ?? UserTierId.STANDARD,\n        userTierName: loadRes.paidTier?.name ?? loadRes.currentTier.name,\n        paidTier: loadRes.paidTier ?? void 0,\n        hasOnboardedPreviously: loadRes.currentTier.hasOnboardedPreviously ?? true\n      };\n    }"
        ),
        (
            "  if (!lroRes.response?.cloudaicompanionProject?.id) {\n    if (projectId) {\n      return {\n        projectId,\n        userTier: tier.id ?? UserTierId.STANDARD,\n        userTierName: tier.name,\n        hasOnboardedPreviously: tier.hasOnboardedPreviously ?? false\n      };\n    }\n    throwIneligibleOrProjectIdError(loadRes);\n  }",
            "  if (!lroRes.response?.cloudaicompanionProject?.id) {\n    return {\n      projectId: projectId || \"\",\n      userTier: tier.id ?? UserTierId.STANDARD,\n      userTierName: tier.name,\n      hasOnboardedPreviously: tier.hasOnboardedPreviously ?? false\n    };\n  }"
        ),
        (
            "getHasAccessToPreviewModel() {\n  return this.hasAccessToPreviewModel ?? false;\n}",
            "getHasAccessToPreviewModel() {\n  return true;\n}"
        ),
        (
            "getGemini31LaunchedSync() {\n    const authType = this.contentGeneratorConfig?.authType;\n    if (this.isGemini31LaunchedForAuthType(authType)) {",
            "getGemini31LaunchedSync() { return true;\n    const authType = this.contentGeneratorConfig?.authType;\n    if (this.isGemini31LaunchedForAuthType(authType)) {"
        ),
        (
            "hasGemini35FlashGAAccess() {\n    const authType = this.contentGeneratorConfig?.authType;\n    const hasAccess = (() => {\n      if (this.isGemini31LaunchedForAuthType(authType)) {",
            "hasGemini35FlashGAAccess() { return true;\n    const authType = this.contentGeneratorConfig?.authType;\n    const hasAccess = (() => {\n      if (this.isGemini31LaunchedForAuthType(authType)) {"
        )
    ];

    let mut patched_count = 0;
    if let Ok(entries) = fs::read_dir(&bundle_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "js") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let mut content_norm = content.replace("\r\n", "\n");
                    let mut changed = false;

                    for (target, replacement) in &patches {
                        let target_norm = target.replace("\r\n", "\n");
                        let replacement_norm = replacement.replace("\r\n", "\n");
                        if content_norm.contains(&target_norm) {
                            content_norm = content_norm.replace(&target_norm, &replacement_norm);
                            changed = true;
                        }
                    }

                    if changed {
                        if fs::write(&path, &content_norm).is_ok() {
                            patched_count += 1;
                        }
                    }
                }
            }
        }
    }

    println!("OK: пропатчено файлов: {}", patched_count);
    Ok(())
}
