
use std::sync::mpsc;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Message {
	None,
	SelectNextVariable(mpsc::Sender<Response>, Option<String>), // optional prefix
	IncrementSelectedVariable(mpsc::Sender<Response>, i32),
	SetVariable(mpsc::Sender<Response>, String, String),
	IncrementVariable(mpsc::Sender<Response>, String, i32),
	SetElementVisibilityByName(String, bool),
	ListElementInstances(mpsc::Sender<Response>),
	GotoNextPage(mpsc::Sender<Response>),
	GotoPrevPage(mpsc::Sender<Response>),
	GotoPage(mpsc::Sender<Response>, usize),
	GotoPageName(mpsc::Sender<Response>, String),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Response {
	None,
	NotImplemented(String),
	ElementInstanceList(String),
	PageChanged(Option<usize>, Option<usize>), // new page #, old page #
	VariableSelected(String),
	VariableChanged(String, f32),
	VariableU32Changed(String, u32),
	VariableF32Changed(String, f32),
	VariableStringChanged(String, String),
}
