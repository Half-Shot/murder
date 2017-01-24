#[derive(Debug, PartialEq, Clone)]
pub enum PlayerRole {
    Unassigned,
    Civilian,
    Mafia,
    Detective,
}

impl PlayerRole {
    pub fn to_string(&self) -> String {
        match self {
            &PlayerRole::Unassigned => "Unassigned".to_string(),
            &PlayerRole::Civilian => "Civilian".to_string(),
            &PlayerRole::Mafia => "Mafia".to_string(),
            &PlayerRole::Detective => "Detective".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    name: String,
    role: PlayerRole,
    is_ghost: bool,
}

impl Player {
    pub fn new(name: String) -> Player {
        Player {
            name: name,
            role: PlayerRole::Unassigned,
            is_ghost: false,
        }
    }

    pub fn assign_role(&mut self, role : PlayerRole) {
        self.role = role;
    }

    pub fn role(&self) -> &PlayerRole{
        return &self.role;
    }

    pub fn name(&self) -> &String{
        return &self.name;
    }

    pub fn is_ghost(&self) -> bool {
        return self.is_ghost;
    }

    pub fn kill(&mut self) {
        self.is_ghost = true;
    }
}
