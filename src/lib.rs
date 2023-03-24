pub mod manifest;

#[cfg(test)]
mod tests {
    use crate::manifest::{self, DiskDrivers, ImageType, NetDrivers};
    #[allow(unused_imports)]
    use crate::manifest::{Manifest, ManifestBuilder};

    #[cfg(feature = "long_tests")]
    static IMGAPI_PUBLIC_SERVER_LIST_URL: &str = "https://images.smartos.org/images";
    #[test]
    #[cfg(feature = "long_tests")]
    fn test_manifest_parsing() {
        let resp = reqwest::blocking::get(IMGAPI_PUBLIC_SERVER_LIST_URL).unwrap();
        let images: Vec<Manifest> = resp.json().unwrap();
        println!("NAME\tVERSION\tUUID\tIMAGE TYPE\tPUBLISHED AT");
        for image in images {
            let published_at = if let Some(published_at) = image.published_at {
                published_at.to_string()
            } else {
                "None".into()
            };
            println!(
                "{}\t{}\t{}\t{}\t{}",
                image.name, image.version, image.uuid, image.image_type, published_at
            );
        }
    }

    #[test]
    fn test_manifest_builder_simple() -> miette::Result<()> {
        let m = ManifestBuilder::default()
            .name("test_manifest")
            .version("v1.0")
            .build()?;

        assert_eq!(m.name, String::from("test_manifest"));
        assert_eq!(m.version, String::from("v1.0"));

        let vm_props = manifest::ImageVMPropertiesBuilder::default()
            .nic_driver(NetDrivers::Virtio)
            .disk_driver(DiskDrivers::Virtio)
            .cpu_type("default")
            .image_size(0)
            .build()?;

        let m2 = ManifestBuilder::default()
            .name("blubber")
            .version("0.1.0")
            .image_type(ImageType::Zvol)
            .vm_image_properties(vm_props)
            .build()?;
        assert_eq!(m2.image_type, ImageType::Zvol);
        assert_eq!(
            m2.vm_image_properties.unwrap().nic_driver.to_string(),
            String::from("virtio")
        );

        Ok(())
    }
}
