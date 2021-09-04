#[derive(yew::Properties, Clone)]
pub struct IconButtonProps {
    pub image: &'static str,
    pub disabled: bool
}

pub struct IconButton {
    image: &'static str,
    disabled: bool,
    ripple_ref: yew::NodeRef,
    on_mousedown: yew::Callback<yew::MouseEvent>
}

impl yew::Component for IconButton {
    type Message = ();
    type Properties = IconButtonProps;

    fn create(props: Self::Properties, _: yew::ComponentLink<Self>) -> Self {
        let ripple_ref = yew::NodeRef::default();
        IconButton {
            image: props.image,
            disabled: props.disabled,
            ripple_ref: ripple_ref.clone(),
            on_mousedown: (move |_| {
                if let Some(elem) = ripple_ref.cast::<web_sys::HtmlElement>() {
                    elem.set_class_name("");
                    elem.offset_height();
                    elem.set_class_name("ripple");
                }
            }).into()
        }
    }

    fn update(&mut self, _: Self::Message) -> yew::ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> yew::ShouldRender {
        self.image = props.image;
        self.disabled = props.disabled;
        true
    }

    fn view(&self) -> yew::Html {
        let class_name = if self.disabled { "disabled" } else { "" };
        return yew::html! {
            <button class=class_name onmousedown=self.on_mousedown.clone()>
                <img src=self.image />
                <span ref=self.ripple_ref.clone() />
            </button>
        };
    }
}