use std::{
    fs::File,
    io::{self, prelude::*, BufReader, Read, Write},
};

struct Contact {
    name: String,
    phone: String,
}

type AppState = Vec<Contact>;

const PAGE_SIZE: usize = 20;
const DEFAULT_FILE_NAME: &str = "agenda.txt";

fn pause() -> io::Result<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    writeln!(stdout)?;
    writeln!(stdout)?;
    write!(stdout, "Pressione qualquer tecla para continuar...")?;

    stdout.flush()?;

    let _ = stdin.read(&mut [0u8])?;

    Ok(())
}

fn clear() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn read_option(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);

    io::stdout().flush()?;

    let mut op = String::new();

    io::stdin().read_line(&mut op)?;

    Ok(op.replace('\n', ""))
}

fn print_contact(contact: &Contact) {
    println!("Name: {}", contact.name);
    println!("Phone: {}", contact.phone);
    println!("--------------------------------------------------");
}

fn exist(app_state: &AppState, name: &str, phone: &str) -> bool {
    let mut exist = false;

    for contact in app_state {
        if contact.name == name && contact.phone == phone {
            exist = true;
            break;
        }
    }

    exist
}

fn read_phone() -> io::Result<String> {
    'phone_loop: loop {
        clear();

        let phone = read_option("Digite o numero do contato: ")?.replace(['(', ')', '-', ' '], "");

        if phone.chars().count() < 11 {
            println!("O telefone deve conter no mínimo 11 dígitos");
            pause()?;
            continue;
        }

        for char in phone.chars() {
            if char != '0'
                && char != '1'
                && char != '2'
                && char != '3'
                && char != '4'
                && char != '5'
                && char != '6'
                && char != '7'
                && char != '8'
                && char != '9'
            {
                println!("O telefone deve conter somente números.");
                pause()?;
                continue 'phone_loop;
            }
        }

        return Ok(phone);
    }
}

fn find_contact(contacts: &AppState, prompt: bool) -> io::Result<Option<usize>> {
    loop {
        clear();

        let name = read_option("Digite o nome a ser pesquisado: ")?;
        let mut found_indexes: Vec<usize> = Vec::new();

        println!("Resultados: ");
        println!("---------------------------------------------------");

        for (index, contact) in contacts.iter().enumerate() {
            if contact.name.starts_with(&name) {
                found_indexes.push(index);
                println!("{}) ", found_indexes.len());
                print_contact(contact);
            }
        }

        if found_indexes.is_empty() {
            println!("Nenhum contato foi encontrado.");
        }

        if !prompt {
            pause()?;
            return Ok(None);
        }

        println!();
        println!("(S)air");

        loop {
            let op = read_option("Digite sua opção: ")?;

            if op == "S" {
                return Ok(None);
            }

            let contact_choosen = op.parse::<usize>();

            if let Ok(contact_choosen) = contact_choosen {
                if let Some(contact_index) = found_indexes.get(contact_choosen - 1) {
                    return Ok(Some(*contact_index));
                }
            }

            println!("Opção inválida.");
        }
    }
}

fn list(contacts: &AppState) -> io::Result<()> {
    if contacts.is_empty() {
        println!("lista vazia.");
        pause()?;
        return Ok(());
    }

    let mut page = 0;

    let page_numbers = ((contacts.len() as f32 / PAGE_SIZE as f32).ceil()) as usize;

    loop {
        clear();

        let current_record = page * PAGE_SIZE;
        let last_record = current_record + PAGE_SIZE;

        for i in current_record..last_record {
            if let Some(contact) = contacts.get(i) {
                print_contact(contact);
            }
        }

        if page > 0 {
            print!("(1) Página Anterior");
        }

        print!("   (2) Sair   ");

        if page < page_numbers - 1 {
            print!("(3) Próxima Página");
        }

        println!();

        let op = read_option("Digite sua opção: ")?;

        match op.as_str() {
            "1" => page = page.saturating_sub(1),
            "3" => {
                if page < PAGE_SIZE {
                    page += 1;
                }
            }
            "2" => break,
            _ => {}
        }
    }

    Ok(())
}

fn search(contacts: &AppState) -> io::Result<()> {
    find_contact(contacts, false)?;

    Ok(())
}

fn create(app_state: &mut AppState) -> io::Result<()> {
    loop {
        clear();

        let name = read_option("Digite o nome do contato: ")?;
        let phone = read_phone()?;

        if exist(app_state, &name, &phone) {
            println!("O contato informado já existe, digite um contato novo.");
            pause()?;
            continue;
        }

        let new_contact = Contact { name, phone };

        app_state.push(new_contact);

        break;
    }

    Ok(())
}

fn update(contacts: &mut AppState) -> io::Result<()> {
    let contact_choosen = find_contact(contacts, true)?;

    loop {
        if let Some(contact_choosen) = contact_choosen {
            if let Some(contact) = contacts.get_mut(contact_choosen) {
                let new_name = read_option(format!("Digite o nome: ({}) ", contact.name).as_str())?;
                let new_phone = read_phone()?;

                if !new_name.trim().is_empty() {
                    contact.name = new_name;
                }

                if !new_phone.trim().is_empty() {
                    contact.phone = new_phone;
                }

                break;
            }
        }

        println!("Opção inválida.");
    }
    Ok(())
}

fn delete(contacts: &mut AppState) -> io::Result<()> {
    clear();

    let contact_choosen = find_contact(contacts, true)?;

    if let Some(contact_choosen) = contact_choosen {
        contacts.remove(contact_choosen);
        println!("Contato removido com sucesso.");
    } else {
        println!("Contato não encontrado.");
    }

    pause()?;

    Ok(())
}

fn load_file(contacts: &mut AppState) -> io::Result<()> {
    clear();

    let mut file_name = read_option("Digite o nome do arquivo (agenda.txt): ")?;

    if file_name.is_empty() {
        file_name = DEFAULT_FILE_NAME.to_string();
    }

    let file = File::open(file_name)?;

    let buffer = BufReader::new(file);

    for line in buffer.lines().flatten() {
        let contact_fields = line.split(',').collect::<Vec<&str>>();

        if let Some(name) = contact_fields.first() {
            if let Some(phone) = contact_fields.get(1) {
                let contact = Contact {
                    name: (*name).to_string(),
                    phone: (*phone).to_string(),
                };

                contacts.push(contact);
            }
        }
    }

    Ok(())
}

fn write_file(contacts: &AppState) -> io::Result<()> {
    clear();

    let mut file_name = read_option("Digite o nome do arquivo (agenda.txt): ")?;

    if file_name.is_empty() {
        file_name = DEFAULT_FILE_NAME.to_string();
    }

    let mut file = File::create(&file_name)?;

    for contact in contacts {
        let line = format!("{},{}\n", contact.name, contact.phone);

        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

fn menu() {
    println!("==================================================");
    println!("AGENDA");
    println!("==================================================");
    println!("1) Listar todos contatos");
    println!("2) Pesquisar contato por nome");
    println!("3) Cadastrar contato");
    println!("4) Alterar contato");
    println!("5) Apagar contato");
    println!("6) Carregar arquivo");
    println!("7) Descarregar arquivo");
    println!("8) Sair");
    println!("==================================================");
    println!();
}

fn event_loop(app_state: &mut AppState) -> io::Result<()> {
    loop {
        clear();

        menu();

        let op = read_option("Digite sua opção > ")?;

        match op.as_str() {
            "1" => list(app_state)?,
            "2" => search(app_state)?,
            "3" => create(app_state)?,
            "4" => update(app_state)?,
            "5" => delete(app_state)?,
            "6" => load_file(app_state)?,
            "7" => write_file(app_state)?,
            "8" => break,
            _ => {}
        };
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut app_state: AppState = vec![];

    event_loop(&mut app_state)?;

    Ok(())
}
