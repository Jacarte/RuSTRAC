use crate::dtw::{Accesor, TokenID};
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

pub struct MMapWrapper {
    pub(crate) name: String,
    // True if the mapped file is temporary
    pub(crate) tmp: bool,
    pub size: usize,
    pub ptr: Arc<Mutex<*mut u8>>,
}

// This only works for UNIX like systems
// It is faster than the naive vectorization of the tokens :)
impl Accesor for MMapWrapper {
    fn get(&self, idx: usize) -> TokenID {
        // get 8 bytes from the pointer
        let ptr = unsafe { self.ptr.lock().unwrap().add(12 + idx * 8) } as *mut TokenID;
        unsafe { ptr.read() }
    }

    fn size(&self) -> usize {
        self.size
    }

    fn get_half(&self) -> Box<dyn Accesor> {
        let halfname = format!("{}.2.bin", self.name);
        // Create a tmp file with half of the size
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(halfname.clone())
            .unwrap();

        let size = self.size / 2;

        log::info!("Copying {} elements", size);
        for i in 0..size{
            //println!("{} {}", i, self.size);
            let element = self.get(i*2);
            file.write_all(element.to_le_bytes().as_ref()).unwrap();
        }


        let ptr = unsafe {
            rustix::mm::mmap(
                std::ptr::null_mut(),
                self.size / 2 * 8,
                rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
                rustix::mm::MapFlags::PRIVATE,
                &file,
                0,
            )
            .expect(&format!("mmap failed to allocate {:#x} bytes", self.size / 2 * 8))
        } as *mut TokenID;
        Box::new(MMapWrapper {
            name: halfname,
            size,
            tmp: true,
            ptr: Arc::new(Mutex::new(ptr as *mut u8)),
        })
    }

}

impl Drop for MMapWrapper {
    fn drop(&mut self) {
        // Unmap the file
        if self.tmp {
            std::fs::remove_file(self.name.clone()).expect("File could not be removed");
        }
    }
}


#[cfg(test)]
mod tests {
    use std::{path::PathBuf, io::Write};

    use crate::{parsing::TraceEncoder, dtw::{Accesor, TokenID}};


    #[test]
    pub fn test_mmap() {
        let  encoder = crate::parsing::ToMemoryParser::default();
        let r1 = encoder.deserialize(PathBuf::from("tests/t1.txt.trace.bin"));

        // Test half getting
        let _r2: Box<dyn Accesor> = r1.get_half();
    }



    #[test]
    pub fn test_larger_files() {
        // Create a super large file
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(String::from("large.bin"))
            .unwrap();
        
        println!("Writing data");
        let size = 1000000000;
        // Write up to 10Gb
        for i in 0..size{
            let b: usize = i;
            file.write_all(b.to_le_bytes().as_ref()).unwrap();
        }

        let ptr = unsafe {
            rustix::mm::mmap(
                std::ptr::null_mut(),
                size * 8,
                rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
                rustix::mm::MapFlags::PRIVATE,
                &file,
                0,
            )
            .expect(&format!("mmap failed to allocate {:#x} bytes", size * 8))
        } as *mut TokenID;

        // Get element by element
        for i in 0..size{
            let ptr = unsafe { ptr.add(i) } as *mut TokenID;
            let element = unsafe { ptr.read() };
            assert_eq!(element, i);
        }
    }

}