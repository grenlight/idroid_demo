use glsl_to_spirv::ShaderType;
use std::error::Error;
use std::fs::read_to_string;
use std::io::Read;
use std::path::PathBuf;

// 参考： https://falseidolfactory.com/2018/06/23/compiling-glsl-to-spirv-at-build-time.html
// 所有 GL_ 打头的宏名称都是 glsl 保留的，不能自定义
const SHADER_VERSION_GL: &str = "#version 450\n";
const SHADER_IMPORT: &str = "#include ";

fn main() -> Result<(), Box<Error>> {
    let shader_files: Vec<&str> = vec![
        //"filter/gaussian_blur",
        "procedual/brick",
    ];

    // Tell the build script to only run again if we change our source shaders
    println!("cargo:rerun-if-changed=shader");

    // Create destination path if necessary
    std::fs::create_dir_all("shader-gen")?;
    for name in shader_files {
        generate_shader_spirv(name, ShaderType::Vertex);
        generate_shader_spirv(name, ShaderType::Fragment);
    }

    Ok(())
}

fn generate_shader_spirv(name: &str, ty: ShaderType) -> Result<(), Box<Error>> {
    let suffix = match ty {
        ShaderType::Vertex => "vs",
        ShaderType::Fragment => "fs",
        _ => "comp",
    };

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("shader")
        .join(format!("{}.{}", name, suffix));
    let mut out_path = "shader-gen/".to_string();
    out_path += &format!("{}_{}.spv", (name.to_string().replace("/", "_")), suffix);

    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            if ty == ShaderType::Vertex {
                load_common_vertex_shader()
            } else {
                panic!("Unable to read {:?}: {:?}", path, e)
            }
        }
    };

    let mut shader_source = String::new();
    shader_source.push_str(SHADER_VERSION_GL);
    parse_shader_source(&code, &mut shader_source);
    // panic!("--panic--");

    let mut output = glsl_to_spirv::compile(&shader_source, ty).unwrap();
    let mut spv = Vec::new();
    output.read_to_end(&mut spv).unwrap();
    let _ =
        std::fs::File::create(&std::path::Path::new(&env!("CARGO_MANIFEST_DIR")).join(&out_path))
            .unwrap();
    std::fs::write(&out_path, &spv).unwrap();

    println!("{:?}", &spv);

    Ok(())
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
        match line.find("//") {
            Some(_) => (),
            None => {
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
                    // output.push_str("\n");
                }
            }
        }
    }
    println!("line: {:?}", output);
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

static VS_MICROS: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shader/func/vs_micros.glsl"));
static FS_MICROS: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shader/func/fs_micros.glsl"));

static COLOR_SPACE_CONVERT: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shader/func/color_space_convert.glsl"));
