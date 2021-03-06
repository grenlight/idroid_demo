use std::fs::read_to_string;
use std::io::Read;
use std::path::PathBuf;

#[cfg(not(target_os = "ios"))]
extern crate shaderc;
use shaderc::ShaderKind;

// 所有 GL_ 打头的宏名称都是 glsl 保留的，不能自定义
const SHADER_VERSION_GL: &str = "#version 450\n";
const SHADER_IMPORT: &str = "#include ";

#[allow(dead_code)]
pub enum ShaderStage {
    General,
    Compute,
}

pub struct Shader {
    pub vs_module: wgpu::ShaderModule,
    pub fs_module: Option<wgpu::ShaderModule>,
}

#[allow(dead_code)]
impl Shader {
    pub fn new(name: &str, device: &mut wgpu::Device) -> Self {
        let (vs_module, fs_module) = load_general_glsl(name, device);
        Shader { vs_module, fs_module: Some(fs_module) }
    }

    // 计算着色
    #[cfg(target_os = "ios")]
    #[allow(dead_code)]
    pub fn new_by_compute(name: &str, device: &mut wgpu::Device) -> Self {
        let bytes = generate_shader_source(name, "comp");
        Shader::shader_by_bytes(&bytes, device)
    }

    #[cfg(not(target_os = "ios"))]
    #[allow(dead_code)]
    pub fn new_by_compute(name: &str, device: &mut wgpu::Device) -> Self {
        let binary_result = generate_shader_source(name, ShaderKind::Compute);
        Shader::shader_by_bytes(binary_result.as_binary_u8(), device)
    }

    fn shader_by_bytes(bytes: &[u8], device: &mut wgpu::Device) -> Self {
        let module = device.create_shader_module(bytes);
        Shader { vs_module: module, fs_module: None }
    }

    pub fn vertex_stage(&self) -> wgpu::PipelineStageDescriptor {
        wgpu::PipelineStageDescriptor { module: &self.vs_module, entry_point: "main" }
    }

    pub fn cs_stage(&self) -> wgpu::PipelineStageDescriptor {
        wgpu::PipelineStageDescriptor { module: &self.vs_module, entry_point: "main" }
    }

    pub fn fragment_stage(&self) -> Option<wgpu::PipelineStageDescriptor> {
        match &self.fs_module {
            Some(fs_module) => {
                Some(wgpu::PipelineStageDescriptor { module: fs_module, entry_point: "main" })
            }
            None => None,
        }
    }
}

#[cfg(target_os = "ios")]
#[allow(dead_code)]
pub fn load_general_glsl(
    name: &str, device: &mut wgpu::Device,
) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vs_bytes = generate_shader_source(name, "vs");
    let fs_bytes = generate_shader_source(name, "fs");
    let vs_module = device.create_shader_module(&vs_bytes);
    let fs_module = device.create_shader_module(&fs_bytes);

    (vs_module, fs_module)
}

#[cfg(target_os = "ios")]
#[allow(dead_code)]
fn generate_shader_source(name: &str, suffix: &str) -> Vec<u8> {
    let p = uni_view::fs::FileSystem::get_shader_path(name, suffix);
    println!("spv path: {:?}", &p);
    let mut f = std::fs::File::open(p).unwrap();
    let mut spv = Vec::new();
    // read the whole file
    f.read_to_end(&mut spv).unwrap();
    spv
}

#[cfg(not(target_os = "ios"))]
#[allow(dead_code)]
pub fn load_general_glsl(
    name: &str, device: &mut wgpu::Device,
) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vs_binary = generate_shader_source(name, ShaderKind::Vertex);
    let fs_binary = generate_shader_source(name, ShaderKind::Fragment);
    let vs_module = device.create_shader_module(vs_binary.as_binary_u8());
    let fs_module = device.create_shader_module(fs_binary.as_binary_u8());

    (vs_module, fs_module)
}

#[cfg(not(target_os = "ios"))]
fn generate_shader_source(name: &str, ty: ShaderKind) -> shaderc::CompilationArtifact {
    let suffix = match ty {
        ShaderKind::Vertex => ".vs",
        ShaderKind::Fragment => ".fs",
        _ => ".comp",
    };

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("shader")
        .join(format!("{}{}", name, suffix));

    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            if cfg!(target_os = "macos") && ty == ShaderKind::Vertex {
                load_common_vertex_shader()
            } else {
                panic!("Unable to read {:?}: {:?}", path, e)
            }
        }
    };
    let mut shader_source = String::new();
    shader_source.push_str(SHADER_VERSION_GL);
    parse_shader_source(&code, &mut shader_source);

    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    let binary_result = compiler.compile_into_spirv(
        &shader_source, ty, "shader.glsl", "main", Some(&options)).unwrap();
    let binary_result2 = compiler.compile_into_spirv_assembly(
        &shader_source, ty, "shader.glsl", "main", Some(&options)).unwrap();

    println!("spirv to text:\n ");

    for line in binary_result2.as_text().lines() {
        println!("{:?}", line );
    }


    binary_result
}

fn load_common_vertex_shader() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shader").join("common.vs");

    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };

    code
}

// Parse a shader string for imports. Imports are recursively processed, and
// prepended to the list of outputs.
fn parse_shader_source(source: &str, output: &mut String) {
    for line in source.lines() {
        if line.starts_with(SHADER_IMPORT) {
            let imports = line[SHADER_IMPORT.len()..].split(',');
            // For each import, get the source, and recurse.
            for import in imports {
                if let Some(include) = get_shader_funcs(import) {
                    parse_shader_source(include, output);
                } else {
                    println!("shader parse error -------");
                    println!("can't find shader functions: {}", import);
                    println!("--------------------------");
                }
            }
        } else {
            output.push_str(line);
            output.push_str("\n");
        }
        // 移除注释
        // match line.find("//") {
        //     Some(_) => (),
        //     None => {

        //     }
        // }
    }
}

// 获取通用 shader function
// 将着色器代码预先静态加载进程序，避免打包成 .a 静态库时找不到文件
fn get_shader_funcs(key: &str) -> Option<&str> {
    match key {
        "color_space_convert" => Some(COLOR_SPACE_CONVERT),
        "vs_micros" => Some(VS_MICROS),
        "fs_micros" => Some(FS_MICROS),
        _ => None,
    }
}

#[allow(dead_code)]
static VS_MICROS: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shader/func/vs_micros.glsl"));

#[allow(dead_code)]
static FS_MICROS: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shader/func/fs_micros.glsl"));

#[allow(dead_code)]
static COLOR_SPACE_CONVERT: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shader/func/color_space_convert.glsl"));
