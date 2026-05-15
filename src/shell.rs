pub fn substitute_args(template: &str, args: &[String]) -> String {
    let mut ret = template.to_string();
    for (index, arg) in args.into_iter().enumerate() {
        ret = ret.replace(&format!("${}", index + 1).to_string(), arg);
    }

    return ret;
}
