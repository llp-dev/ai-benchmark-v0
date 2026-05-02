NAME = push_swap

SRC = push_swap.rs

all: $(NAME)

$(NAME): $(SRC)
	rustc --edition 2021 --crate-type bin -C opt-level=2 -D warnings -o $(NAME) $(SRC)

clean:
	rm -f $(NAME)

fclean: clean

re: fclean all

.PHONY: all clean fclean re
