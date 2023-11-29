import React from "react";

export const Chat = ({ chatMessages }) => {
	return (
		<div>
			{chatMessages.slice().map((chatMessage, index) => (
				<div
					key={index}
					className={`mb-8 ${index % 2 === 0 ? "text-right" : "text-left"}`}
				>
					{index % 2 === 0 && (
						<span className="font-bold text-docViolet text-xl">You</span>
					)}
					{index % 2 !== 0 && (
						<span className="font-bold text-docGreen text-xl">Jon</span>
					)}
					<br />
					<span>{chatMessage}</span>
				</div>
			))}
			<style jsx>{`
				.text-left {
					text-align: left;
				}
				.text-right {
					text-align: right;
				}
			`}</style>
		</div>
	);
};
