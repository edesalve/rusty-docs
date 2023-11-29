import React, { useState } from "react";

export const UserMsg = ({ addChatMessage, sendUserQuestion }) => {
	const [inputValue, setInputValue] = useState("");
	const MIN_HEIGHT = 48;
	const MAX_HEIGHT = 180;
	const [textareaHeight, setTextareaHeight] = useState(MIN_HEIGHT);

	const handleSendMessage = async () => {
		if (inputValue.trim() !== "") {
			addChatMessage(inputValue);
			setInputValue("");
			setTextareaHeight(MIN_HEIGHT);
		}
		await sendUserQuestion(inputValue);
	};

	return (
		<div className="bg-docGrey4 flex relative">
			<textarea
				className="border border-1 border-docGrey2 flex-auto focus:outline-none focus:border-docGrey1 pl-2 pr-12 py-2 placeholder-docGrey1 overflow-auto resize-none rounded scrollable-content text-lg"
				placeholder="Message Jon..."
				value={inputValue}
				onChange={(e) => {
					setInputValue(e.target.value);
					const newHeight = Math.min(
						MAX_HEIGHT,
						Math.max(MIN_HEIGHT, e.target.scrollHeight)
					);
					setTextareaHeight(newHeight);
				}}
				style={{
					backgroundColor: "inherit",
					height: `${textareaHeight}px`,
				}}
			/>
			<button
				className="absolute bg-docWhite bottom-2 cursor-pointer disabled:bg-docGrey4 disabled:text-docGrey1 ml-2 p-2 right-2 rounded text-docGrey3"
				onClick={handleSendMessage}
				disabled={inputValue.trim() === ""}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 22 22"
					strokeWidth="2.5"
					stroke="currentColor"
					className="w-4 h-4"
				>
					<path
						strokeLinecap="round"
						strokeLinejoin="round"
						d="M12 19.5v-15m0 0l-6.75 6.75M12 4.5l6.75 6.75"
					/>
				</svg>
			</button>
			<style jsx>
				{`
					.scrollable-content::-webkit-scrollbar {
						display: none;
					}
				`}
			</style>
		</div>
	);
};
