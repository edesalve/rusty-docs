import React from "react";

export const Header = () => {
	return (
		<header className=" bg-docGrey4 flex flex-row items-center justify-between px-8">
			<img src={`rusty_docs.svg`} className="w-auto h-10 my-2" alt="Logo" />
			rusty-docs-0.1.0
		</header>
	);
};
