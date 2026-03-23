use image::DynamicImage;
use lru::LruCache;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct ArtworkCache {
    cache_dir: PathBuf,
    memory_cache: Arc<Mutex<LruCache<String, DynamicImage>>>,
}

impl Clone for ArtworkCache {
    fn clone(&self) -> Self {
        Self {
            cache_dir: self.cache_dir.clone(),
            memory_cache: Arc::clone(&self.memory_cache),
        }
    }
}

impl ArtworkCache {
    pub fn new(cache_dir: PathBuf, capacity: usize) -> Self {
        Self {
            cache_dir,
            memory_cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("cache capacity must be non-zero"),
            ))),
        }
    }

    pub async fn get(&self, url: &str) -> Option<DynamicImage> {
        {
            let mut cache = self.memory_cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(img) = cache.get(url) {
                return Some(img.clone());
            }
        }

        let hash = self.hash_url(url);
        let path = self.cache_dir.join(format!("{}.png", hash));

        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            let path_clone = path.clone();
            if let Ok(Ok(img)) = tokio::task::spawn_blocking(move || image::open(path_clone)).await
            {
                let mut cache = self.memory_cache.lock().unwrap_or_else(|e| e.into_inner());
                cache.put(url.to_string(), img.clone());
                return Some(img);
            }
        }

        None
    }

    pub async fn insert(&self, url: String, img: DynamicImage) {
        let hash = self.hash_url(&url);
        let path = self.cache_dir.join(format!("{}.png", hash));

        if !tokio::fs::try_exists(&self.cache_dir)
            .await
            .unwrap_or(false)
        {
            tokio::fs::create_dir_all(&self.cache_dir).await.ok();
        }

        let img_clone = img.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(mut file) = std::fs::File::create(path) {
                let _ = img_clone.write_to(&mut file, image::ImageFormat::Png);
            }
        })
        .await
        .ok();

        let mut cache = self.memory_cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.put(url, img);
    }

    fn hash_url(&self, url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url);
        format!("{:x}", hasher.finalize())
    }
}
