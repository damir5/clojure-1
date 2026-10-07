use std::{collections::HashMap, fs};
use zed_extension_api::{self as zed, LanguageServerId, Result};

const CLOJURE_LSP: ToolSpec = ToolSpec {
    repo: "clojure-lsp/clojure-lsp",
    binary: "clojure-lsp",
    asset_prefix: "clojure-lsp-native",
};

struct ToolSpec {
    repo: &'static str,
    binary: &'static str,
    asset_prefix: &'static str,
}

impl ToolSpec {
    /// The GitHub release asset name for the given platform.
    fn asset_name(&self, platform: zed::Os, arch: zed::Architecture) -> Result<String> {
        Ok(format!(
            "{}-{os}-{arch}.zip",
            self.asset_prefix,
            os = match platform {
                zed::Os::Mac => "macos",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X8664 => "amd64",
                zed::Architecture::X86 =>
                    return Err(format!("unsupported architecture: {arch:?}")),
            },
        ))
    }

    /// The (version directory, binary path) a release of `version`
    /// unpacks to inside the extension's working directory.
    fn install_paths(&self, version: &str, platform: zed::Os) -> (String, String) {
        let version_dir = format!("{}-{version}", self.binary);
        let suffix = if platform == zed::Os::Windows {
            ".exe"
        } else {
            ""
        };
        let binary_path = format!("{version_dir}/{}{suffix}", self.binary);
        (version_dir, binary_path)
    }

    fn is_outdated_version(&self, name: &str, current_version: &str) -> bool {
        name.starts_with(&format!("{}-", self.binary)) && name != current_version
    }
}

/// The outside world the binary-resolution policy runs against: the
/// host platform, GitHub release metadata, the extension working
/// directory and the extension API's downloader. Production wires the
/// real Zed API and filesystem; tests wire fakes so the resolution
/// order itself is exercised without network or disk.
trait Env {
    fn platform(&mut self) -> (zed::Os, zed::Architecture);
    fn latest_release(&mut self, repo: &str) -> Result<zed::GithubRelease>;
    fn is_file(&mut self, path: &str) -> bool;
    fn download_zip(&mut self, url: &str, dir: &str) -> Result<()>;
    fn make_executable(&mut self, path: &str) -> Result<()>;
    /// Names of the directories in the extension working directory.
    fn work_dirs(&mut self) -> Result<Vec<String>>;
    fn remove_dir(&mut self, name: &str) -> Result<()>;
}

/// Production Env: the Zed extension API and the real filesystem.
/// The working directory is the extension's own data directory.
struct ZedEnv<'a> {
    language_server_id: &'a LanguageServerId,
}

impl Env for ZedEnv<'_> {
    fn platform(&mut self) -> (zed::Os, zed::Architecture) {
        zed::current_platform()
    }

    fn latest_release(&mut self, repo: &str) -> Result<zed::GithubRelease> {
        zed::set_language_server_installation_status(
            self.language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        zed::latest_github_release(
            repo,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
    }

    fn is_file(&mut self, path: &str) -> bool {
        fs::metadata(path).is_ok_and(|stat| stat.is_file())
    }

    fn download_zip(&mut self, url: &str, dir: &str) -> Result<()> {
        zed::set_language_server_installation_status(
            self.language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );
        zed::download_file(url, dir, zed::DownloadedFileType::Zip)
            .map_err(|e| format!("failed to download file: {e}"))
    }

    fn make_executable(&mut self, path: &str) -> Result<()> {
        zed::make_file_executable(path)
    }

    fn work_dirs(&mut self) -> Result<Vec<String>> {
        Ok(fs::read_dir(".")
            .map_err(|e| format!("failed to list working directory {e}"))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect())
    }

    fn remove_dir(&mut self, name: &str) -> Result<()> {
        fs::remove_dir_all(name).map_err(|e| format!("failed to remove {name:?}: {e}"))
    }
}

struct ClojureExtension {
    cached_binary_paths: HashMap<String, String>,
}

impl ClojureExtension {
    /// Resolution order: a binary found on the worktree PATH, then a
    /// previously resolved path that still exists, then the latest
    /// GitHub release downloaded into a per-version directory (removing
    /// outdated versions of the same tool). An explicitly configured
    /// binary wins over all of this; the caller handles that case.
    fn resolve_binary_path(
        &mut self,
        env: &mut dyn Env,
        id: &str,
        path_binary: Option<String>,
        tool: &ToolSpec,
    ) -> Result<String> {
        if let Some(path) = path_binary {
            return Ok(path);
        }

        if let Some(path) = self.cached_binary_paths.get(id) {
            if env.is_file(path) {
                return Ok(path.clone());
            }
        }

        let (platform, arch) = env.platform();
        let release = env.latest_release(tool.repo)?;
        let asset_name = tool.asset_name(platform, arch)?;
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {asset_name:?}"))?;
        let (version_dir, binary_path) = tool.install_paths(&release.version, platform);

        if !env.is_file(&binary_path) {
            env.download_zip(&asset.download_url, &version_dir)?;
            env.make_executable(&binary_path)?;
            for name in env.work_dirs()? {
                if tool.is_outdated_version(&name, &version_dir) {
                    env.remove_dir(&name)?;
                }
            }
        }

        self.cached_binary_paths.insert(id.to_string(), binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for ClojureExtension {
    fn new() -> Self {
        Self {
            cached_binary_paths: HashMap::new(),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let tool = match language_server_id.as_ref() {
            "clojure-lsp" => &CLOJURE_LSP,
            id => return Err(format!("unsupported language server: {id}")),
        };

        // An explicitly configured binary wins over PATH and downloads.
        if let Some(binary) =
            zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree)?
                .binary
        {
            if let Some(path) = binary.path {
                return Ok(zed::Command {
                    command: path,
                    args: binary.arguments.unwrap_or_default(),
                    env: binary.env.unwrap_or_default().into_iter().collect(),
                });
            }
        }

        Ok(zed::Command {
            command: self.resolve_binary_path(
                &mut ZedEnv { language_server_id },
                language_server_id.as_ref(),
                worktree.which(tool.binary),
                tool,
            )?,
            args: Vec::new(),
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings =
            zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings.initialization_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings =
            zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings.settings)
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: zed::lsp::Completion,
    ) -> Option<zed::CodeLabel> {
        let mut spans = vec![zed::CodeLabelSpan::literal(&completion.label, None)];
        if let Some(detail) = completion.detail {
            spans.push(zed::CodeLabelSpan::literal(
                format!(" {detail}"),
                Some("comment".to_string()),
            ));
        }
        Some(zed::CodeLabel {
            code: completion.label.clone(),
            spans,
            filter_range: (0..completion.label.len()).into(),
        })
    }
}

zed::register_extension!(ClojureExtension);

#[cfg(test)]
mod tests { // @fdb:lsp-binary-resolution
    use super::*;
    use zed::Extension;

    const MAC_ARM: (zed::Os, zed::Architecture) = (zed::Os::Mac, zed::Architecture::Aarch64);

    struct FakeEnv {
        files: Vec<String>,
        dirs: Vec<String>,
        downloads: Vec<String>,
        removed: Vec<String>,
        release_calls: usize,
    }

    impl FakeEnv {
        fn new() -> Self {
            Self {
                files: Vec::new(),
                dirs: Vec::new(),
                downloads: Vec::new(),
                removed: Vec::new(),
                release_calls: 0,
            }
        }
    }

    impl Env for FakeEnv {
        fn platform(&mut self) -> (zed::Os, zed::Architecture) {
            MAC_ARM
        }

        fn latest_release(&mut self, _repo: &str) -> Result<zed::GithubRelease> {
            self.release_calls += 1;
            Ok(zed::GithubRelease {
                version: "v9.9.9".to_string(),
                assets: vec![zed::GithubReleaseAsset {
                    name: "clojure-lsp-native-macos-aarch64.zip".to_string(),
                    download_url: "https://example.com/clojure-lsp.zip".to_string(),
                }],
            })
        }

        fn is_file(&mut self, path: &str) -> bool {
            self.files.iter().any(|f| f == path)
        }

        fn download_zip(&mut self, url: &str, dir: &str) -> Result<()> {
            self.downloads.push(url.to_string());
            self.files.push(format!("{dir}/clojure-lsp"));
            self.dirs.push(dir.to_string());
            Ok(())
        }

        fn make_executable(&mut self, _path: &str) -> Result<()> {
            Ok(())
        }

        fn work_dirs(&mut self) -> Result<Vec<String>> {
            Ok(self.dirs.clone())
        }

        fn remove_dir(&mut self, name: &str) -> Result<()> {
            self.dirs.retain(|d| d != name);
            self.removed.push(name.to_string());
            Ok(())
        }
    }

    fn resolve(env: &mut impl Env, ext: &mut ClojureExtension, path: Option<&str>) -> Result<String> {
        ext.resolve_binary_path(
            env,
            "clojure-lsp",
            path.map(str::to_string),
            &CLOJURE_LSP,
        )
    }

    #[test]
    fn path_binary_wins_without_release_lookup() {
        let mut env = FakeEnv::new();
        let mut ext = ClojureExtension::new();
        let path = resolve(&mut env, &mut ext, Some("/opt/homebrew/bin/clojure-lsp")).unwrap();
        assert_eq!(path, "/opt/homebrew/bin/clojure-lsp");
        assert_eq!(env.release_calls, 0);
    }

    #[test]
    fn in_memory_cache_short_circuits_when_file_exists() {
        let mut env = FakeEnv::new();
        env.files.push("clojure-lsp-v1.0.0/clojure-lsp".to_string());
        let mut ext = ClojureExtension::new();
        ext.cached_binary_paths
            .insert("clojure-lsp".to_string(), "clojure-lsp-v1.0.0/clojure-lsp".to_string());
        let path = resolve(&mut env, &mut ext, None).unwrap();
        assert_eq!(path, "clojure-lsp-v1.0.0/clojure-lsp");
        assert_eq!(env.release_calls, 0);
    }

    #[test]
    fn missing_binary_downloads_and_cleans_outdated_versions() {
        let mut env = FakeEnv::new();
        env.dirs.push("clojure-lsp-v1.0.0".to_string());
        env.dirs.push("unrelated".to_string());
        let mut ext = ClojureExtension::new();

        let path = resolve(&mut env, &mut ext, None).unwrap();
        assert_eq!(path, "clojure-lsp-v9.9.9/clojure-lsp");
        assert_eq!(env.downloads, vec!["https://example.com/clojure-lsp.zip"]);
        assert_eq!(env.removed, vec!["clojure-lsp-v1.0.0"]);
        assert!(env.dirs.contains(&"unrelated".to_string()));
        assert_eq!(
            ext.cached_binary_paths.get("clojure-lsp").unwrap(),
            "clojure-lsp-v9.9.9/clojure-lsp"
        );
    }

    #[test]
    fn existing_download_skips_download_and_cleanup() {
        let mut env = FakeEnv::new();
        env.files.push("clojure-lsp-v9.9.9/clojure-lsp".to_string());
        env.dirs.push("clojure-lsp-v1.0.0".to_string());
        let mut ext = ClojureExtension::new();

        let path = resolve(&mut env, &mut ext, None).unwrap();
        assert_eq!(path, "clojure-lsp-v9.9.9/clojure-lsp");
        assert!(env.downloads.is_empty());
        assert!(env.removed.is_empty());
    }

    #[test]
    fn missing_asset_is_an_error() {
        struct NoAssets;
        impl Env for NoAssets {
            fn platform(&mut self) -> (zed::Os, zed::Architecture) {
                MAC_ARM
            }
            fn latest_release(&mut self, _repo: &str) -> Result<zed::GithubRelease> {
                Ok(zed::GithubRelease {
                    version: "v9.9.9".to_string(),
                    assets: Vec::new(),
                })
            }
            fn is_file(&mut self, _path: &str) -> bool {
                false
            }
            fn download_zip(&mut self, _url: &str, _dir: &str) -> Result<()> {
                unreachable!("no asset to download")
            }
            fn make_executable(&mut self, _path: &str) -> Result<()> {
                unreachable!()
            }
            fn work_dirs(&mut self) -> Result<Vec<String>> {
                Ok(Vec::new())
            }
            fn remove_dir(&mut self, _name: &str) -> Result<()> {
                unreachable!()
            }
        }

        let mut env = NoAssets;
        let mut ext = ClojureExtension::new();
        let err = resolve(&mut env, &mut ext, None).unwrap_err();
        assert!(err.contains("no asset found matching"));
    }

    #[test]
    fn unsupported_architecture_is_an_error() {
        let err = CLOJURE_LSP
            .asset_name(zed::Os::Mac, zed::Architecture::X86)
            .unwrap_err();
        assert!(err.contains("unsupported architecture"));
    }

    #[test]
    fn cleanup_preserves_other_tools_and_current_version() {
        let tool = ToolSpec {
            repo: "",
            binary: "clojure-lsp",
            asset_prefix: "",
        };
        let current = "clojure-lsp-new";
        assert!(tool.is_outdated_version("clojure-lsp-old", current));
        assert!(!tool.is_outdated_version(current, current));
        assert!(!tool.is_outdated_version("clj-kondo-old", current));
        assert!(!tool.is_outdated_version("unrelated", current));
    }
}
