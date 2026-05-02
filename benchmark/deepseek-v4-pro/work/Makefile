NAME = push_swap

all: $(NAME)

$(NAME): push_swap.rs
	rustc --edition 2021 --crate-type bin -C opt-level=2 -D warnings -o $(NAME) push_swap.rs

clean:
	rm -f $(NAME)

fclean: clean

re: fclean all

.PHONY: all clean fclean re