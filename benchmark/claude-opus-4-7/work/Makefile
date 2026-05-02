NAME = push_swap
SRC  = push_swap.rs
RUSTC = rustc
RUSTFLAGS = --edition 2021 --crate-type bin -C opt-level=2 -D warnings

all: $(NAME)

$(NAME): $(SRC)
	$(RUSTC) $(RUSTFLAGS) -o $(NAME) $(SRC)

clean:
	rm -f $(NAME).d

fclean: clean
	rm -f $(NAME)

re: fclean all

.PHONY: all clean fclean re
